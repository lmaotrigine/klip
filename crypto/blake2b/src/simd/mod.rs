#![allow(non_camel_case_types)]

mod ops;
mod opt;

#[cfg(feature = "simd_opt")]
macro_rules! transmute_shuffle {
    ($tmp:ident, $n:expr, $vec:expr, $idx:expr) => {
        unsafe {
            const IDX: [u32; $n] = $idx;
            let tmp_i = ::core::mem::transmute::<_, $tmp>($vec);
            let tmp_o = $crate::simd::ops::simd_shuffle::<_, _, $tmp>(tmp_i, tmp_i, IDX);
            ::core::mem::transmute(tmp_o)
        }
    };
}
#[cfg(feature = "simd_opt")]
use transmute_shuffle;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "simd", repr(simd))]
#[cfg_attr(not(feature = "simd"), repr(C))]
pub struct u64x4(pub u64, pub u64, pub u64, pub u64);

#[cfg_attr(not(feature = "simd"), allow(clippy::missing_const_for_fn))]
impl u64x4 {
    #[inline(always)]
    pub const fn new(e0: u64, e1: u64, e2: u64, e3: u64) -> Self {
        Self(e0, e1, e2, e3)
    }

    #[inline(always)]
    pub const fn gather(src: &[u64], i0: usize, i1: usize, i2: usize, i3: usize) -> Self {
        Self::new(src[i0], src[i1], src[i2], src[i3])
    }

    #[cfg(target_endian = "little")]
    #[allow(clippy::wrong_self_convention)]
    #[inline(always)]
    pub const fn from_le(self) -> Self {
        self
    }

    #[cfg(not(target_endian = "little"))]
    #[allow(clippy::wrong_self_convention)]
    #[inline(always)]
    pub const fn from_le(self) -> Self {
        Self::new(
            u64::from_le(self.0),
            u64::from_le(self.1),
            u64::from_le(self.2),
            u64::from_le(self.3),
        )
    }

    #[cfg(target_endian = "little")]
    #[inline(always)]
    pub const fn to_le(self) -> Self {
        self
    }

    #[cfg(not(target_endian = "little"))]
    #[inline(always)]
    pub const fn to_le(self) -> Self {
        Self::new(
            self.0.to_le(),
            self.1.to_le(),
            self.2.to_le(),
            self.3.to_le(),
        )
    }

    #[inline(always)]
    pub fn wrapping_add(self, rhs: Self) -> Self {
        self + rhs
    }

    #[inline(always)]
    pub fn rotate_right(self, n: u32) -> Self {
        opt::rotate_right(self, n)
    }

    #[cfg(feature = "simd")]
    #[inline(always)]
    pub fn shuffle_left_1(self) -> Self {
        const IDX: [u32; 4] = [1, 2, 3, 0];
        unsafe { ops::simd_shuffle(self, self, IDX) }
    }

    #[cfg(feature = "simd")]
    #[inline(always)]
    pub fn shuffle_left_2(self) -> Self {
        const IDX: [u32; 4] = [2, 3, 0, 1];
        unsafe { ops::simd_shuffle(self, self, IDX) }
    }

    #[cfg(feature = "simd")]
    #[inline(always)]
    pub fn shuffle_left_3(self) -> Self {
        const IDX: [u32; 4] = [3, 0, 1, 2];
        unsafe { ops::simd_shuffle(self, self, IDX) }
    }

    #[cfg(not(feature = "simd"))]
    #[inline(always)]
    pub const fn shuffle_left_1(self) -> Self {
        Self::new(self.1, self.2, self.3, self.0)
    }

    #[cfg(not(feature = "simd"))]
    #[inline(always)]
    pub const fn shuffle_left_2(self) -> Self {
        Self::new(self.2, self.3, self.0, self.1)
    }

    #[cfg(not(feature = "simd"))]
    #[inline(always)]
    pub const fn shuffle_left_3(self) -> Self {
        Self::new(self.3, self.0, self.1, self.2)
    }

    #[inline(always)]
    pub fn shuffle_right_1(self) -> Self {
        self.shuffle_left_3()
    }

    #[inline(always)]
    pub fn shuffle_right_2(self) -> Self {
        self.shuffle_left_2()
    }

    #[inline(always)]
    pub fn shuffle_right_3(self) -> Self {
        self.shuffle_left_1()
    }
}

#[cfg(feature = "simd_opt")]
#[derive(Debug, Clone, Copy)]
#[repr(simd)]
pub struct u8x32(
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
    pub u8,
);

#[cfg(feature = "simd_opt")]
#[derive(Debug, Clone, Copy)]
#[repr(simd)]
pub struct u16x16(
    pub u16,
    pub u16,
    pub u16,
    pub u16,
    pub u16,
    pub u16,
    pub u16,
    pub u16,
    pub u16,
    pub u16,
    pub u16,
    pub u16,
    pub u16,
    pub u16,
    pub u16,
    pub u16,
);

#[cfg(feature = "simd_opt")]
#[derive(Debug, Clone, Copy)]
#[repr(simd)]
pub struct u32x8(
    pub u32,
    pub u32,
    pub u32,
    pub u32,
    pub u32,
    pub u32,
    pub u32,
    pub u32,
);
