use super::u64x4;
use core::ops::{Add, BitXor, Shl, Shr};

#[cfg(feature = "simd")]
extern "platform-intrinsic" {
    pub fn simd_add<T>(x: T, y: T) -> T;
    pub fn simd_shl<T>(x: T, y: T) -> T;
    pub fn simd_shr<T>(x: T, y: T) -> T;
    pub fn simd_xor<T>(x: T, y: T) -> T;
    pub fn simd_shuffle<T, U, V>(v: T, w: T, idx: U) -> V;
}

impl Add for u64x4 {
    type Output = Self;

    #[cfg(feature = "simd")]
    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        unsafe { simd_add(self, rhs) }
    }

    #[cfg(not(feature = "simd"))]
    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(
            self.0.wrapping_add(rhs.0),
            self.1.wrapping_add(rhs.1),
            self.2.wrapping_add(rhs.2),
            self.3.wrapping_add(rhs.3),
        )
    }
}

impl BitXor for u64x4 {
    type Output = Self;

    #[cfg(feature = "simd")]
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self::Output {
        unsafe { simd_xor(self, rhs) }
    }

    #[cfg(not(feature = "simd"))]
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::new(
            self.0 ^ rhs.0,
            self.1 ^ rhs.1,
            self.2 ^ rhs.2,
            self.3 ^ rhs.3,
        )
    }
}

impl Shl<Self> for u64x4 {
    type Output = Self;

    #[cfg(feature = "simd")]
    #[inline(always)]
    fn shl(self, rhs: Self) -> Self::Output {
        unsafe { simd_shl(self, rhs) }
    }

    #[cfg(not(feature = "simd"))]
    #[inline(always)]
    fn shl(self, rhs: Self) -> Self::Output {
        Self::new(
            self.0 << rhs.0,
            self.1 << rhs.1,
            self.2 << rhs.2,
            self.3 << rhs.3,
        )
    }
}

impl Shr<Self> for u64x4 {
    type Output = Self;

    #[cfg(feature = "simd")]
    #[inline(always)]
    fn shr(self, rhs: Self) -> Self::Output {
        unsafe { simd_shr(self, rhs) }
    }

    #[cfg(not(feature = "simd"))]
    #[inline(always)]
    fn shr(self, rhs: Self) -> Self::Output {
        Self::new(
            self.0 >> rhs.0,
            self.1 >> rhs.1,
            self.2 >> rhs.2,
            self.3 >> rhs.3,
        )
    }
}
