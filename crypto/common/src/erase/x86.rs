use super::{atomic_fence, volatile_write, Erase};

#[cfg(target_arch = "x86")]
use core::arch::x86::{__m128, __m128d, __m128i, __m256, __m256d, __m256i};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{__m128, __m128d, __m128i, __m256, __m256d, __m256i};

macro_rules! impl_erase_for_simd_register {
    ($($name:ty),+ $(,)?) => {
        $(
            impl Erase for $name {
                #[inline]
                fn erase(&mut self) {
                    volatile_write(unsafe { core::mem::zeroed() }, self);
                    atomic_fence();
                }
            }
        )+
    };
}

impl_erase_for_simd_register!(__m128, __m128d, __m128i, __m256, __m256d, __m256i);
