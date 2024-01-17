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
    clippy::nursery
)]
#![allow(clippy::inline_always)]

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "aarch64")]
use aarch64::compress;
mod consts;
mod soft;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
use soft::compress;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use x86::compress;

type Block = crypto_common::blocks::Block<128>;
type Buffer = crypto_common::blocks::Buffer<128>;

#[allow(missing_copy_implementations)]
#[derive(Clone)]
struct Core {
    state: [u64; 8],
    block_len: u128,
}

impl Default for Core {
    fn default() -> Self {
        Self {
            state: consts::H,
            block_len: 0,
        }
    }
}

impl Core {
    #[inline]
    fn update_blocks(&mut self, blocks: &[Block]) {
        self.block_len += blocks.len() as u128;
        compress(&mut self.state, blocks);
    }

    #[inline]
    fn finalize(&mut self, buffer: &mut Buffer, out: &mut [u8; 64]) {
        let bit_len = 8 * (buffer.get_pos() as u128 + self.block_len * 128);
        buffer.len128_padding_be(bit_len, |b| {
            compress(&mut self.state, core::slice::from_ref(b));
        });
        for (chunk, v) in out.chunks_exact_mut(8).zip(self.state.iter()) {
            chunk.copy_from_slice(&v.to_be_bytes());
        }
    }
}

#[derive(Clone)]
pub struct Sha512 {
    core: Core,
    buffer: Buffer,
}

impl Default for Sha512 {
    #[inline]
    fn default() -> Self {
        Self {
            core: Core::default(),
            buffer: Buffer::default(),
        }
    }
}

impl core::fmt::Debug for Sha512 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Sha512 { ... }")
    }
}

impl Sha512 {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn update(&mut self, input: &[u8]) {
        let Self { core, buffer } = self;
        buffer.digest_blocks(input, |blocks| core.update_blocks(blocks));
    }

    #[inline]
    pub fn finalize(&mut self) -> [u8; 64] {
        let mut out = [0; 64];
        let Self { core, buffer } = self;
        core.finalize(buffer, &mut out);
        out
    }

    #[inline]
    #[must_use]
    pub fn digest(data: &[u8]) -> [u8; 64] {
        let mut hasher = Core::default();
        let mut buffer = Buffer::default();
        buffer.digest_blocks(data, |b| hasher.update_blocks(b));
        let mut out = [0; 64];
        hasher.finalize(&mut buffer, &mut out);
        out
    }
}
