use crate::{portable, utils::as_array, BLOCK_BYTES};
use ::core::cmp;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
target_features::detect!(cpuid_avx2, "avx2");
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
target_features::detect!(cpuid_sse41, "sse4.1");

#[allow(clippy::cast_possible_truncation)]
pub const fn count_low(count: u128) -> u64 {
    count as u64
}

#[allow(clippy::cast_possible_truncation)]
pub const fn count_high(count: u128) -> u64 {
    (count >> u64::BITS as usize) as u64
}

pub const fn flag_word(flag: bool) -> u64 {
    if flag {
        !0
    } else {
        0
    }
}

pub fn final_block<'a>(
    input: &'a [u8],
    offset: usize,
    buffer: &'a mut [u8; BLOCK_BYTES],
) -> (&'a [u8; BLOCK_BYTES], usize, bool) {
    let capped_offset = cmp::min(offset, input.len());
    let offset_slice = &input[capped_offset..];
    if offset_slice.len() >= BLOCK_BYTES {
        let block = as_array!(offset_slice, 0, BLOCK_BYTES);
        let should_finalize = offset_slice.len() <= BLOCK_BYTES;
        (block, BLOCK_BYTES, should_finalize)
    } else {
        buffer[..offset_slice.len()].copy_from_slice(offset_slice);
        (buffer, offset_slice.len(), true)
    }
}

pub fn input_assertions(input: &[u8], finalize: bool) {
    if !finalize {
        debug_assert!(!input.is_empty());
        debug_assert_eq!(input.len() % BLOCK_BYTES, 0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(any(target_arch = "x86", target_arch = "x86_64"), allow(dead_code))]
pub enum Platform {
    Portable,
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    SSE41,
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    AVX2,
}

impl Platform {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[cfg_attr(
        target_feature = "avx2",
        allow(unreachable_code, clippy::missing_const_for_fn)
    )]
    fn avx2() -> Option<Self> {
        #[cfg(target_feature = "avx2")]
        {
            return Some(Self::AVX2);
        }
        if cpuid_avx2::get() {
            Some(Self::AVX2)
        } else {
            None
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[cfg_attr(
        target_feature = "sse4.1",
        allow(unreachable_code, clippy::missing_const_for_fn)
    )]
    fn sse41() -> Option<Self> {
        #[cfg(target_feature = "sse4.1")]
        {
            return Some(Self::SSE41);
        }
        if cpuid_sse41::get() {
            Some(Self::SSE41)
        } else {
            None
        }
    }

    #[cfg_attr(
        not(any(target_arch = "x86", target_arch = "x86_64")),
        allow(clippy::missing_const_for_fn)
    )]
    pub fn detect() -> Self {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            Self::avx2().or_else(Self::sse41).unwrap_or(Self::Portable)
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            Self::Portable
        }
    }

    pub fn compress1_loop(
        self,
        input: &[u8],
        words: &mut [u64; 8],
        count: u128,
        last_node: bool,
        finalize: bool,
    ) {
        match self {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Self::AVX2 => unsafe {
                crate::avx2::compress1_loop(input, words, count, last_node, finalize);
            },
            // there is an SSE version of compress1 in the C impl. I'll port it later maybe.
            _ => {
                portable::compress1_loop(input, words, count, last_node, finalize);
            }
        }
    }
}
