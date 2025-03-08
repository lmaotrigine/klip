#![no_std]
#![deny(
    dead_code,
    deprecated,
    future_incompatible,
    missing_copy_implementations,
    missing_debug_implementations,
    nonstandard_style,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unused,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::unwrap_used,
    clippy::alloc_instead_of_core,
    clippy::std_instead_of_alloc,
    clippy::std_instead_of_core
)]
#![allow(clippy::inline_always, clippy::missing_panics_doc)]

use ::core::mem::size_of;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;
mod core;
mod portable;
mod utils;
use utils::{as_arrays, as_arrays_mut};

pub const OUT_BYTES: usize = u64::BITS as usize;
pub const KEY_BYTES: usize = u64::BITS as usize;
pub const SALT_BYTES: usize = 2 * size_of::<u64>();
pub const PERSONAL_BYTES: usize = 2 * size_of::<u64>();
pub const BLOCK_BYTES: usize = 16 * size_of::<u64>();

const IV: [u64; 8] = [
    0x6a09_e667_f3bc_c908,
    0xbb67_ae85_84ca_a73b,
    0x3c6e_f372_fe94_f82b,
    0xa54f_f53a_5f1d_36f1,
    0x510e_527f_ade6_82d1,
    0x9b05_688c_2b3e_6c1f,
    0x1f83_d9ab_fb41_bd6b,
    0x5be0_cd19_137e_2179,
];

const SIGMA: [[u8; 16]; 12] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
    [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
    [6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
    [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
];

fn to_hex(bytes: &[u8], hex: &mut [u8]) {
    let table = b"0123456789abcdef";
    for (i, &b) in bytes.iter().enumerate() {
        hex[i * 2] = table[(b >> 4) as usize];
        hex[i * 2 + 1] = table[(b & 0xf) as usize];
    }
}

#[derive(Clone, Copy)]
pub struct Hash {
    bytes: [u8; OUT_BYTES],
    len: u8,
}

impl Hash {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len as usize]
    }

    #[inline]
    #[must_use]
    pub fn as_array(&self) -> &[u8; OUT_BYTES] {
        debug_assert_eq!(self.len as usize, OUT_BYTES);
        &self.bytes
    }
}

impl From<[u8; OUT_BYTES]> for Hash {
    #[allow(clippy::cast_possible_truncation)]
    fn from(value: [u8; OUT_BYTES]) -> Self {
        Self {
            bytes: value,
            len: OUT_BYTES as u8,
        }
    }
}

impl From<&[u8; OUT_BYTES]> for Hash {
    fn from(value: &[u8; OUT_BYTES]) -> Self {
        Self::from(*value)
    }
}

impl PartialEq for Hash {
    fn eq(&self, other: &Self) -> bool {
        crypto_common::constant_time::ConstantTimeEq::ct_eq(self.as_bytes(), other.as_bytes())
            .to_u8()
            == 1
    }
}

impl PartialEq<[u8]> for Hash {
    fn eq(&self, other: &[u8]) -> bool {
        crypto_common::constant_time::ConstantTimeEq::ct_eq(self.as_bytes(), other).to_u8() == 1
    }
}

impl Eq for Hash {}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl ::core::fmt::Debug for Hash {
    // very slow but meh.
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        let mut out = [0; OUT_BYTES];
        to_hex(self.as_bytes(), &mut out);
        f.write_str("Hash(")?;
        for &b in &out[..self.len as usize * 2] {
            write!(f, "{b:02x}")?;
        }
        f.write_str(")")?;
        Ok(())
    }
}

#[inline(always)]
fn state_words_to_bytes(state_words: &[u64; 8]) -> [u8; OUT_BYTES] {
    const W: usize = size_of::<u64>();
    let mut bytes = [0; OUT_BYTES];
    let arrs = as_arrays_mut!(&mut bytes, W, W, W, W, W, W, W, W);
    *arrs.0 = state_words[0].to_le_bytes();
    *arrs.1 = state_words[1].to_le_bytes();
    *arrs.2 = state_words[2].to_le_bytes();
    *arrs.3 = state_words[3].to_le_bytes();
    *arrs.4 = state_words[4].to_le_bytes();
    *arrs.5 = state_words[5].to_le_bytes();
    *arrs.6 = state_words[6].to_le_bytes();
    *arrs.7 = state_words[7].to_le_bytes();
    bytes
}

#[allow(missing_copy_implementations)]
#[derive(Clone)]
pub struct State {
    words: [u64; 8],
    count: u128,
    buf: [u8; BLOCK_BYTES],
    buf_len: u8,
    last_node: bool,
    hash_length: u8,
    implementation: core::Platform,
    is_keyed: bool,
}

impl State {
    #[must_use]
    pub fn new() -> Self {
        Self::with_params(&Params::new())
    }

    #[allow(clippy::cast_possible_truncation)]
    const fn with_params(params: &Params) -> Self {
        let mut state = Self {
            words: params.to_words(),
            count: 0,
            buf: [0; BLOCK_BYTES],
            buf_len: 0,
            last_node: params.last_node,
            hash_length: params.hash_length,
            implementation: params.implementation,
            is_keyed: params.key_length > 0,
        };
        if state.is_keyed {
            state.buf = params.key_block;
            state.buf_len = state.buf.len() as u8;
        }
        state
    }

    #[allow(clippy::cast_possible_truncation)]
    fn fill_buf(&mut self, input: &mut &[u8]) {
        let take = ::core::cmp::min(BLOCK_BYTES - self.buf_len as usize, input.len());
        self.buf[self.buf_len as usize..self.buf_len as usize + take]
            .copy_from_slice(&input[..take]);
        self.buf_len += take as u8;
        *input = &input[take..];
    }

    fn compress_buffer_if_possible(&mut self, input: &mut &[u8]) {
        if self.buf_len > 0 {
            self.fill_buf(input);
            if !input.is_empty() {
                self.implementation.compress1_loop(
                    &self.buf,
                    &mut self.words,
                    self.count,
                    self.last_node,
                    false,
                );
                self.count = self.count.wrapping_add(BLOCK_BYTES as u128);
                self.buf_len = 0;
            }
        }
    }

    pub fn update(&mut self, mut input: &[u8]) -> &mut Self {
        self.compress_buffer_if_possible(&mut input);
        let mut end = input.len().saturating_sub(1);
        end -= end % BLOCK_BYTES;
        if end > 0 {
            self.implementation.compress1_loop(
                &input[..end],
                &mut self.words,
                self.count,
                self.last_node,
                false,
            );
            self.count = self.count.wrapping_add(end as u128);
            input = &input[end..];
        }
        self.fill_buf(&mut input);
        self
    }

    #[must_use]
    pub fn finalize(&self) -> Hash {
        let mut words_copy = self.words;
        self.implementation.compress1_loop(
            &self.buf[..self.buf_len as usize],
            &mut words_copy,
            self.count,
            self.last_node,
            true,
        );
        Hash {
            bytes: state_words_to_bytes(&words_copy),
            len: self.hash_length,
        }
    }

    #[must_use]
    pub const fn count(&self) -> u128 {
        let mut ret = self.count.wrapping_add(self.buf_len as u128);
        if self.is_keyed {
            ret -= BLOCK_BYTES as u128;
        }
        ret
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl ::core::fmt::Debug for State {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("State")
            .field("count", &self.count())
            .field("hash_length", &self.hash_length)
            .field("last_node", &self.last_node)
            .finish_non_exhaustive()
    }
}

#[allow(missing_copy_implementations)]
#[derive(Clone)]
pub struct Params {
    hash_length: u8,
    key_length: u8,
    key_block: [u8; BLOCK_BYTES],
    salt: [u8; SALT_BYTES],
    personal: [u8; PERSONAL_BYTES],
    fanout: u8,
    max_depth: u8,
    max_leaf_length: u32,
    node_offset: u64,
    node_depth: u8,
    inner_hash_length: u8,
    last_node: bool,
    implementation: core::Platform,
}

impl Params {
    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn new() -> Self {
        Self {
            hash_length: OUT_BYTES as u8,
            key_length: 0,
            key_block: [0; BLOCK_BYTES],
            salt: [0; SALT_BYTES],
            personal: [0; PERSONAL_BYTES],
            fanout: 1,
            max_depth: 1,
            max_leaf_length: 0,
            node_offset: 0,
            node_depth: 0,
            inner_hash_length: 0,
            last_node: false,
            implementation: core::Platform::detect(),
        }
    }

    #[inline(always)]
    #[allow(clippy::cast_lossless)]
    const fn to_words(&self) -> [u64; 8] {
        let (salt_left, salt_right) = as_arrays!(&self.salt, SALT_BYTES / 2, SALT_BYTES / 2);
        let (personal_left, personal_right) =
            as_arrays!(&self.personal, PERSONAL_BYTES / 2, PERSONAL_BYTES / 2);
        [
            IV[0]
                ^ self.hash_length as u64
                ^ ((self.key_length as u64) << 8)
                ^ ((self.fanout as u64) << 16)
                ^ ((self.max_depth as u64) << 24)
                ^ ((self.max_leaf_length as u64) << 32),
            IV[1] ^ self.node_offset,
            IV[2] ^ self.node_depth as u64 ^ ((self.inner_hash_length as u64) << 8),
            IV[3],
            IV[4] ^ u64::from_le_bytes(*salt_left),
            IV[5] ^ u64::from_le_bytes(*salt_right),
            IV[6] ^ u64::from_le_bytes(*personal_left),
            IV[7] ^ u64::from_le_bytes(*personal_right),
        ]
    }

    #[must_use]
    pub const fn to_state(&self) -> State {
        State::with_params(self)
    }

    #[inline]
    #[must_use]
    pub fn hash(&self, input: &[u8]) -> Hash {
        if self.key_length > 0 {
            return self.to_state().update(input).finalize();
        }
        let mut words = self.to_words();
        self.implementation
            .compress1_loop(input, &mut words, 0, self.last_node, true);
        Hash {
            bytes: state_words_to_bytes(&words),
            len: self.hash_length,
        }
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    pub fn hash_length(&mut self, length: usize) -> &mut Self {
        assert!(
            (1..=OUT_BYTES).contains(&length),
            "Bad hash length: {length}"
        );
        self.hash_length = length as u8;
        self
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    pub fn key(&mut self, key: &[u8]) -> &mut Self {
        assert!(key.len() <= KEY_BYTES, "Bad key length: {}", key.len());
        self.key_length = key.len() as u8;
        self.key_block = [0; BLOCK_BYTES];
        self.key_block[..key.len()].copy_from_slice(key);
        self
    }

    #[inline]
    pub fn salt(&mut self, salt: &[u8]) -> &mut Self {
        assert!(salt.len() <= SALT_BYTES, "Bad salt length: {}", salt.len());
        self.salt = [0; SALT_BYTES];
        self.salt[..salt.len()].copy_from_slice(salt);
        self
    }

    #[inline]
    pub fn personal(&mut self, personal: &[u8]) -> &mut Self {
        assert!(
            personal.len() <= PERSONAL_BYTES,
            "Bad personal length: {}",
            personal.len()
        );
        self.personal = [0; PERSONAL_BYTES];
        self.personal[..personal.len()].copy_from_slice(personal);
        self
    }

    #[inline]
    pub const fn fanout(&mut self, fanout: u8) -> &mut Self {
        self.fanout = fanout;
        self
    }

    #[inline]
    pub const fn max_depth(&mut self, max_depth: u8) -> &mut Self {
        self.max_depth = max_depth;
        self
    }

    #[inline]
    pub const fn max_leaf_length(&mut self, max_leaf_length: u32) -> &mut Self {
        self.max_leaf_length = max_leaf_length;
        self
    }

    #[inline]
    pub const fn node_offset(&mut self, node_offset: u64) -> &mut Self {
        self.node_offset = node_offset;
        self
    }

    #[inline]
    pub const fn node_depth(&mut self, node_depth: u8) -> &mut Self {
        self.node_depth = node_depth;
        self
    }

    #[inline]
    pub const fn inner_hash_length(&mut self, inner_hash_length: u8) -> &mut Self {
        self.inner_hash_length = inner_hash_length;
        self
    }

    #[inline]
    pub const fn last_node(&mut self, last_node: bool) -> &mut Self {
        self.last_node = last_node;
        self
    }
}

impl Default for Params {
    fn default() -> Self {
        Self::new()
    }
}

impl ::core::fmt::Debug for Params {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("Params")
            .field("hash_length", &self.hash_length)
            .field("key_length", &self.key_length)
            .field("salt", &self.salt)
            .field("personal", &self.personal)
            .field("fanout", &self.fanout)
            .field("max_depth", &self.max_depth)
            .field("max_leaf_length", &self.max_leaf_length)
            .field("node_offset", &self.node_offset)
            .field("node_depth", &self.node_depth)
            .field("inner_hash_length", &self.inner_hash_length)
            .field("last_node", &self.last_node)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    #[test]
    fn test_working() {
        std::println!(
            "{:?}",
            Params::new().hash_length(32).key(b"foo").hash(b"hello")
        );
    }
}
