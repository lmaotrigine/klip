#![no_std]
#![deny(
    dead_code,
    deprecated,
    future_incompatible,
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

use buffer::MutableBuffer;
use crypto_common::{
    blocks::Block as Block_,
    erase::{Erase, EraseOnDrop},
};

mod backends;
mod buffer;

type XNonce = [u8; 24];
type Block = Block_<64>;

const ROUNDS: usize = 10;
const KEY_SIZE: usize = 32;
const BLOCK_SIZE: usize = 64;

const CONSTANTS: [u32; 4] = [0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574];

type Key = [u8; KEY_SIZE];

#[derive(Debug, Clone, Copy)]
struct Error;

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("xchacha20: fuck.")
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
target_features::detect!(avx2, "avx2");
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
target_features::detect!(sse2, "sse2");

struct Core {
    state: [u32; 16],
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    features: (avx2::Features, sse2::Features),
}

impl Drop for Core {
    fn drop(&mut self) {
        self.state.erase();
    }
}

impl EraseOnDrop for Core {}

impl Core {
    #[inline]
    #[must_use]
    #[allow(clippy::missing_panics_doc)] // never happens
    pub fn new(key: &Key, iv: &XNonce) -> Self {
        let subkey = hchacha20(key, &iv[..16].try_into().unwrap());
        let mut padded_iv = [0; 12];
        padded_iv[4..].copy_from_slice(&iv[16..]);
        let mut state = [0; 16];
        state[..4].copy_from_slice(&CONSTANTS);
        let key_chunks = subkey.chunks_exact(4);
        for (val, chunk) in state[4..12].iter_mut().zip(key_chunks) {
            *val = u32::from_le_bytes(chunk.try_into().unwrap());
        }
        let iv_chunks = padded_iv.chunks_exact(4);
        for (val, chunk) in state[13..16].iter_mut().zip(iv_chunks) {
            *val = u32::from_le_bytes(chunk.try_into().unwrap());
        }
        Self {
            state,
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            features: (avx2::init(), sse2::init()),
        }
    }

    #[inline(always)]
    fn remaining_blocks(&self) -> Option<usize> {
        let rem = u32::MAX - self.state[12];
        rem.try_into().ok()
    }

    fn process_with_backend(&mut self, ctx: impl Context) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if self.features.0.get() {
                unsafe { backends::avx2::inner(&mut self.state, ctx) };
            } else if self.features.1.get() {
                unsafe { backends::sse2::inner(&mut self.state, ctx) };
            } else {
                ctx.call(&mut backends::soft::Backend(self));
            }
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
            {
                unsafe { backends::neon::inner(&mut self.state, ctx) };
            }
            #[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]
            {
                ctx.call(&mut backends::soft::Backend(self));
            }
        }
    }

    #[inline]
    fn write_keystream_block(&mut self, block: &mut Block) {
        self.process_with_backend(WriteContext { block });
    }

    #[inline]
    fn apply_keystream_blocks(&mut self, blocks: MutableBuffer<'_, Block>) {
        self.process_with_backend(ApplyContext { blocks });
    }
}

pub struct XChaCha20 {
    core: Core,
    buffer: Block,
}

impl XChaCha20 {
    #[inline]
    #[must_use]
    pub fn new(key: &Key, iv: &XNonce) -> Self {
        let core = Core::new(key, iv);
        let mut buffer = [0; BLOCK_SIZE];
        buffer[0] = 64;
        Self { core, buffer }
    }

    #[inline]
    fn get_pos(&self) -> u8 {
        let pos = self.buffer[0];
        if pos == 0 || pos > 64 {
            debug_assert!(false);
            unsafe {
                core::hint::unreachable_unchecked();
            }
        }
        pos
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn set_pos(&mut self, pos: usize) {
        debug_assert!(pos != 0 && pos <= 64);
        self.buffer[0] = pos as u8;
    }

    #[inline]
    fn remaining(&self) -> u8 {
        64 - self.get_pos()
    }

    #[allow(clippy::manual_div_ceil)]
    fn check_remaining(&self, data_len: usize) -> Result<(), Error> {
        let Some(rem_blocks) = self.core.remaining_blocks() else {
            return Ok(());
        };
        let buf_rem = usize::from(self.remaining());
        let data_len = match data_len.checked_sub(buf_rem) {
            Some(0) | None => return Ok(()),
            Some(res) => res,
        };
        let blocks = (data_len + 63) / 64;
        if blocks > rem_blocks {
            Err(Error)
        } else {
            Ok(())
        }
    }

    #[inline]
    fn try_apply_keystream_mut(&mut self, mut data: MutableBuffer<'_, u8>) -> Result<(), Error> {
        self.check_remaining(data.len())?;
        let pos = usize::from(self.get_pos());
        let rem = usize::from(self.remaining());
        let data_len = data.len();
        if rem != 0 {
            if data_len <= rem {
                data.xor(&self.buffer[pos..][..data_len]);
                self.set_pos(pos + data_len);
                return Ok(());
            }
            let (mut left, right) = data.split_at(rem);
            data = right;
            left.xor(&self.buffer[pos..]);
        }
        let (blocks, mut tail) = data.into_chunks();
        self.core.apply_keystream_blocks(blocks);
        let new_pos = if tail.is_empty() {
            BLOCK_SIZE
        } else {
            self.core.write_keystream_block(&mut self.buffer);
            tail.xor(&self.buffer.as_ref()[..tail.len()]);
            tail.len()
        };
        self.set_pos(new_pos);
        Ok(())
    }

    /// # Panics
    /// If the end of the keystream will be reached, this will panic without
    /// modifying the buffer.
    pub fn apply_keystream(&mut self, buf: &mut [u8]) {
        self.try_apply_keystream_mut(buf.into()).unwrap();
    }
}

impl Drop for XChaCha20 {
    fn drop(&mut self) {
        self.buffer.erase();
    }
}

impl EraseOnDrop for XChaCha20 {}

impl core::fmt::Debug for XChaCha20 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("XChaCha20")
    }
}

const fn quarter_round(a: usize, b: usize, c: usize, d: usize, state: &mut [u32; 16]) {
    state[a] = state[a].wrapping_add(state[b]);
    state[d] ^= state[a];
    state[d] = state[d].rotate_left(16);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] ^= state[c];
    state[b] = state[b].rotate_left(12);
    state[a] = state[a].wrapping_add(state[b]);
    state[d] ^= state[a];
    state[d] = state[d].rotate_left(8);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] ^= state[c];
    state[b] = state[b].rotate_left(7);
}

fn hchacha20(key: &Key, input: &[u8; 16]) -> [u8; 32] {
    let mut state = [0; 16];
    state[..4].copy_from_slice(&CONSTANTS);
    let key_chunks = key.chunks_exact(4);
    for (v, chunk) in state[4..12].iter_mut().zip(key_chunks) {
        *v = u32::from_le_bytes(chunk.try_into().unwrap());
    }
    let input_chunks = input.chunks_exact(4);
    for (v, chunk) in state[12..16].iter_mut().zip(input_chunks) {
        *v = u32::from_le_bytes(chunk.try_into().unwrap());
    }
    for _ in 0..ROUNDS {
        quarter_round(0, 4, 8, 12, &mut state);
        quarter_round(1, 5, 9, 13, &mut state);
        quarter_round(2, 6, 10, 14, &mut state);
        quarter_round(3, 7, 11, 15, &mut state);
        quarter_round(0, 5, 10, 15, &mut state);
        quarter_round(1, 6, 11, 12, &mut state);
        quarter_round(2, 7, 8, 13, &mut state);
        quarter_round(3, 4, 9, 14, &mut state);
    }
    let mut output = [0; 32];
    for (chunk, val) in output[..16].chunks_exact_mut(4).zip(&state[..4]) {
        chunk.copy_from_slice(&val.to_le_bytes());
    }
    for (chunk, val) in output[16..].chunks_exact_mut(4).zip(&state[12..]) {
        chunk.copy_from_slice(&val.to_le_bytes());
    }
    output
}

trait Context {
    fn call<B: backends::Backend<P>, const P: usize>(self, backend: &mut B);
}

struct ApplyContext<'a> {
    blocks: MutableBuffer<'a, Block>,
}

impl Context for ApplyContext<'_> {
    #[inline(always)]
    #[allow(clippy::needless_range_loop)]
    fn call<B: backends::Backend<P>, const P: usize>(self, backend: &mut B) {
        if P > 1 {
            let (chunks, mut tail) = self.blocks.into_chunks::<P>();
            for mut chunk in chunks {
                let mut tmp = [[0; BLOCK_SIZE]; P];
                backend.gen_par_ks_blocks(&mut tmp);
                chunk.xor(&tmp);
            }
            let n = tail.len();
            let mut buf = [[0; BLOCK_SIZE]; P];
            let ks = &mut buf[..n];
            backend.gen_tail_blocks(ks);
            for i in 0..n {
                tail.get(i).xor(&ks[i]);
            }
        } else {
            for mut block in self.blocks {
                let mut t = [0; BLOCK_SIZE];
                backend.gen_ks_block(&mut t);
                block.xor(&t);
            }
        }
    }
}

struct WriteContext<'a> {
    block: &'a mut Block,
}

impl Context for WriteContext<'_> {
    #[inline(always)]
    fn call<B: backends::Backend<P>, const P: usize>(self, backend: &mut B) {
        backend.gen_ks_block(self.block);
    }
}
