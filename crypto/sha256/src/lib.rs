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
#![cfg_attr(
    any(sha256_backend = "riscv-zknh", sha256_backend = "riscv-zknh-compact"),
    feature(riscv_ext_intrinsics)
)]

use core::fmt::Debug;
use crypto_common::blocks::{Block as Block_, Buffer as Buffer_};

type Block = Block_<64>;
type Buffer = Buffer_<64>;

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "aarch64")]
use aarch64::compress;
mod consts;
mod soft;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;
#[cfg(not(any(
    target_arch = "aarch64",
    target_arch = "x86",
    target_arch = "x86_64",
    all(
        any(target_arch = "riscv32", target_arch = "riscv64"),
        any(sha256_backend = "riscv-zknh", sha256_backend = "riscv-zknh-compact")
    )
)))]
use soft::compress;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use x86::compress;
#[cfg(all(
    any(target_arch = "riscv32", target_arch = "riscv64"),
    sha256_backend = "riscv-zknh"
))]
mod riscv_zknh;
#[cfg(all(
    any(target_arch = "riscv32", target_arch = "riscv64"),
    sha256_backend = "riscv-zknh"
))]
use riscv_zknh::compress;
#[cfg(all(
    any(target_arch = "riscv32", target_arch = "riscv64"),
    sha256_backend = "riscv-zknh-compact"
))]
mod riscv_zknh_compact;
#[cfg(all(
    any(target_arch = "riscv32", target_arch = "riscv64"),
    sha256_backend = "riscv-zknh-compact"
))]
use riscv_zknh_compact::compress;
#[cfg(all(
    any(target_arch = "riscv32", target_arch = "riscv64"),
    any(sha256_backend = "riscv-zknh", sha256_backend = "riscv-zknh-compact")
))]
mod riscv_zknh_utils;

#[cfg(all(
    any(sha256_backend = "riscv-zknh", sha256_backend = "riscv-zknh-compact"),
    not(any(target_arch = "riscv32", target_arch = "riscv64"))
))]
compile_error!("zknh backends can only be enabled for RISC-V targets");

#[allow(missing_copy_implementations)]
#[derive(Clone)]
pub struct Sha256 {
    state: [u32; 8],
    block_len: u64,
}

impl Default for Sha256 {
    fn default() -> Self {
        Self {
            state: consts::H,
            block_len: 0,
        }
    }
}

impl Sha256 {
    #[inline]
    pub fn update_blocks(&mut self, blocks: &[Block]) {
        self.block_len += blocks.len() as u64;
        compress(&mut self.state, blocks);
    }

    #[inline]
    pub fn finalize(&mut self, buffer: &mut Buffer, out: &mut [u8; 32]) {
        let bit_len = 8 * (buffer.get_pos() as u64 + self.block_len * 64);
        buffer.len64_padding_be(bit_len, |b| {
            compress(&mut self.state, core::slice::from_ref(b));
        });
        for (chunk, v) in out.chunks_exact_mut(4).zip(self.state.iter()) {
            chunk.copy_from_slice(&v.to_be_bytes());
        }
    }

    #[inline]
    #[must_use]
    pub fn digest(data: &[u8]) -> [u8; 32] {
        let mut hasher = Self::default();
        let mut buffer = Buffer::default();
        buffer.digest_blocks(data, |b| hasher.update_blocks(b));
        let mut out = [0; 32];
        hasher.finalize(&mut buffer, &mut out);
        out
    }
}

impl Debug for Sha256 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Sha256 { ... }")
    }
}
