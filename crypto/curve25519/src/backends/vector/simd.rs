use core::ops::{Add, AddAssign, BitAnd, BitAndAssign, BitXor, BitXorAssign, Sub};

macro_rules! impl_shared {
    ($ty:ident, $lane_ty:ident, $add:ident, $sub:ident, $shl:ident, $shr:ident, $extract:ident) => {
        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone, Copy)]
        #[repr(transparent)]
        pub struct $ty(::core::arch::x86_64::__m256i);

        #[::macros::target_feature("avx2")]
        impl From<$ty> for ::core::arch::x86_64::__m256i {
            #[inline]
            fn from(value: $ty) -> Self {
                value.0
            }
        }

        #[::macros::target_feature("avx2")]
        impl From<::core::arch::x86_64::__m256i> for $ty {
            #[inline]
            fn from(value: ::core::arch::x86_64::__m256i) -> Self {
                Self(value)
            }
        }

        #[::macros::target_feature("avx2")]
        impl PartialEq for $ty {
            #[inline]
            fn eq(&self, rhs: &Self) -> bool {
                unsafe {
                    let m = ::core::arch::x86_64::_mm256_cmpeq_epi8(self.0, rhs.0);
                    ::core::arch::x86_64::_mm256_movemask_epi8(m) == -1
                }
            }
        }

        impl Eq for $ty {}

        #[::macros::target_feature("avx2")]
        impl Add for $ty {
            type Output = Self;

            #[inline]
            fn add(self, rhs: Self) -> Self {
                unsafe { ::core::arch::x86_64::$add(self.0, rhs.0).into() }
            }
        }

        #[allow(clippy::assign_op_pattern)]
        #[::macros::target_feature("avx2")]
        impl AddAssign for $ty {
            #[inline]
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs;
            }
        }

        #[::macros::target_feature("avx2")]
        impl Sub for $ty {
            type Output = Self;

            #[inline]
            fn sub(self, rhs: Self) -> Self {
                unsafe { ::core::arch::x86_64::$sub(self.0, rhs.0).into() }
            }
        }

        #[::macros::target_feature("avx2")]
        impl BitAnd for $ty {
            type Output = Self;

            #[inline]
            fn bitand(self, rhs: Self) -> Self {
                unsafe { ::core::arch::x86_64::_mm256_and_si256(self.0, rhs.0).into() }
            }
        }

        #[allow(clippy::assign_op_pattern)]
        #[::macros::target_feature("avx2")]
        impl BitAndAssign for $ty {
            #[inline]
            fn bitand_assign(&mut self, rhs: Self) {
                *self = *self & rhs;
            }
        }

        #[::macros::target_feature("avx2")]
        impl BitXor for $ty {
            type Output = Self;

            #[inline]
            fn bitxor(self, rhs: Self) -> Self {
                unsafe { ::core::arch::x86_64::_mm256_xor_si256(self.0, rhs.0).into() }
            }
        }

        #[allow(clippy::assign_op_pattern)]
        #[::macros::target_feature("avx2")]
        impl BitXorAssign for $ty {
            #[inline]
            fn bitxor_assign(&mut self, rhs: Self) {
                *self = *self ^ rhs;
            }
        }

        #[::macros::target_feature("avx2")]
        impl $ty {
            #[inline]
            pub fn shl<const N: i32>(self) -> Self {
                unsafe { ::core::arch::x86_64::$shl(self.0, N).into() }
            }

            #[inline]
            pub fn shr<const N: i32>(self) -> Self {
                unsafe { ::core::arch::x86_64::$shr(self.0, N).into() }
            }

            #[inline]
            pub fn extract<const N: i32>(self) -> $lane_ty {
                unsafe { ::core::arch::x86_64::$extract(self.0, N) as $lane_ty }
            }
        }
    };
}

macro_rules! impl_conv {
    ($($src:ident => $dst:ident),+) => {
        $(
            #[::macros::target_feature("avx2")]
            impl From<$src> for $dst {
                #[inline]
                fn from(value: $src) -> Self {
                    Self(value.0)
                }
            }
        )+
    };
}

impl_shared!(
    u64x4,
    u64,
    _mm256_add_epi64,
    _mm256_sub_epi64,
    _mm256_slli_epi64,
    _mm256_srli_epi64,
    _mm256_extract_epi64
);
impl_shared!(
    u32x8,
    u32,
    _mm256_add_epi32,
    _mm256_sub_epi32,
    _mm256_slli_epi32,
    _mm256_srli_epi32,
    _mm256_extract_epi32
);
impl_conv!(u64x4 => u32x8);

#[allow(clippy::cast_possible_wrap, clippy::too_many_arguments)]
impl u32x8 {
    #[inline]
    pub const fn new_const(
        x0: u32,
        x1: u32,
        x2: u32,
        x3: u32,
        x4: u32,
        x5: u32,
        x6: u32,
        x7: u32,
    ) -> Self {
        unsafe { Self(core::mem::transmute([x0, x1, x2, x3, x4, x5, x6, x7])) }
    }

    #[inline]
    pub const fn splat_const<const N: u32>() -> Self {
        Self::new_const(N, N, N, N, N, N, N, N)
    }

    #[macros::target_feature("avx2")]
    #[inline]
    pub fn new(x0: u32, x1: u32, x2: u32, x3: u32, x4: u32, x5: u32, x6: u32, x7: u32) -> u32x8 {
        unsafe {
            u32x8(::core::arch::x86_64::_mm256_set_epi32(
                x7 as i32, x6 as i32, x5 as i32, x4 as i32, x3 as i32, x2 as i32, x1 as i32,
                x0 as i32,
            ))
        }
    }

    #[macros::target_feature("avx2")]
    #[inline]
    pub fn splat(x: u32) -> u32x8 {
        unsafe { u32x8(::core::arch::x86_64::_mm256_set1_epi32(x as i32)) }
    }
}

#[allow(clippy::cast_possible_wrap)]
impl u64x4 {
    #[inline]
    pub const fn new_const(x0: u64, x1: u64, x2: u64, x3: u64) -> Self {
        unsafe { Self(core::mem::transmute([x0, x1, x2, x3])) }
    }

    #[inline]
    pub const fn splat_const<const N: u64>() -> Self {
        Self::new_const(N, N, N, N)
    }

    #[macros::target_feature("avx2")]
    #[inline]
    pub fn new(x0: u64, x1: u64, x2: u64, x3: u64) -> u64x4 {
        unsafe {
            u64x4(core::arch::x86_64::_mm256_set_epi64x(
                x3 as i64, x2 as i64, x1 as i64, x0 as i64,
            ))
        }
    }

    #[macros::target_feature("avx2")]
    #[inline]
    pub fn splat(x: u64) -> u64x4 {
        unsafe { u64x4(core::arch::x86_64::_mm256_set1_epi64x(x as i64)) }
    }
}

#[macros::target_feature("avx2")]
impl u32x8 {
    #[inline]
    pub fn mul32(self, rhs: Self) -> u64x4 {
        unsafe { core::arch::x86_64::_mm256_mul_epu32(self.0, rhs.0).into() }
    }
}
