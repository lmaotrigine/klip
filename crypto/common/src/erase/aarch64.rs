use super::{atomic_fence, volatile_write, Erase};
use core::arch::aarch64::{
    uint16x4_t, uint16x8_t, uint32x2_t, uint32x4_t, uint64x1_t, uint64x2_t, uint8x16_t, uint8x8_t,
};

macro_rules! impl_erase_for_simd_register {
    ($($name:ty),+$(,)?) => {
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

impl_erase_for_simd_register!(
    uint8x8_t, uint8x16_t, uint16x4_t, uint16x8_t, uint32x2_t, uint32x4_t, uint64x1_t, uint64x2_t,
);
