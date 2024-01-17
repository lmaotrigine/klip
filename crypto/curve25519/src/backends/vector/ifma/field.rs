use core::ops::{Add, Mul, Neg};

use crate::backends::{serial::u64::field::FieldElement51, vector::simd::u64x4};

#[macros::target_feature("avx512ifma,avx512vl")]
#[inline]
unsafe fn madd52lo(z: u64x4, x: u64x4, y: u64x4) -> u64x4 {
    core::arch::x86_64::_mm256_madd52lo_epu64(z.into(), x.into(), y.into()).into()
}

#[macros::target_feature("avx512ifma,avx512vl")]
#[inline]
unsafe fn madd52hi(z: u64x4, x: u64x4, y: u64x4) -> u64x4 {
    core::arch::x86_64::_mm256_madd52hi_epu64(z.into(), x.into(), y.into()).into()
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub enum Shuffle {
    AAAA,
    BBBB,
    BADC,
    BACD,
    ADDA,
    CBCB,
    ABDC,
    ABAB,
    DBBD,
    CACA,
}

#[macros::target_feature("avx512ifma,avx512vl")]
#[inline(always)]
fn shuffle_lanes(x: u64x4, control: Shuffle) -> u64x4 {
    unsafe {
        use core::arch::x86_64::_mm256_permute4x64_epi64 as perm;
        match control {
            Shuffle::AAAA => perm(x.into(), 0b00_00_00_00).into(),
            Shuffle::BBBB => perm(x.into(), 0b01_01_01_01).into(),
            Shuffle::BADC => perm(x.into(), 0b10_11_00_01).into(),
            Shuffle::BACD => perm(x.into(), 0b11_10_00_01).into(),
            Shuffle::ADDA => perm(x.into(), 0b00_11_11_00).into(),
            Shuffle::CBCB => perm(x.into(), 0b01_10_01_10).into(),
            Shuffle::ABDC => perm(x.into(), 0b10_11_01_00).into(),
            Shuffle::ABAB => perm(x.into(), 0b01_00_01_00).into(),
            Shuffle::DBBD => perm(x.into(), 0b11_01_01_11).into(),
            Shuffle::CACA => perm(x.into(), 0b00_10_00_10).into(),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub enum Lanes {
    D,
    C,
    AB,
    AC,
    AD,
    BCD,
}

#[macros::target_feature("avx512ifma,avx512vl")]
#[inline]
fn blend_lanes(x: u64x4, y: u64x4, control: Lanes) -> u64x4 {
    unsafe {
        use core::arch::x86_64::_mm256_blend_epi32 as blend;
        match control {
            Lanes::D => blend(x.into(), y.into(), 0b11_00_00_00).into(),
            Lanes::C => blend(x.into(), y.into(), 0b00_11_00_00).into(),
            Lanes::AB => blend(x.into(), y.into(), 0b00_00_11_11).into(),
            Lanes::AC => blend(x.into(), y.into(), 0b00_11_00_11).into(),
            Lanes::AD => blend(x.into(), y.into(), 0b11_00_00_11).into(),
            Lanes::BCD => blend(x.into(), y.into(), 0b11_11_11_00).into(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct F51x4Unreduced(pub(crate) [u64x4; 5]);

#[macros::target_feature("avx512ifma,avx512vl")]
impl F51x4Unreduced {
    pub const ZERO: Self = Self([u64x4::splat_const::<0>(); 5]);

    pub fn new(
        x0: &FieldElement51,
        x1: &FieldElement51,
        x2: &FieldElement51,
        x3: &FieldElement51,
    ) -> Self {
        Self([
            u64x4::new(x0.0[0], x1.0[0], x2.0[0], x3.0[0]),
            u64x4::new(x0.0[1], x1.0[1], x2.0[1], x3.0[1]),
            u64x4::new(x0.0[2], x1.0[2], x2.0[2], x3.0[2]),
            u64x4::new(x0.0[3], x1.0[3], x2.0[3], x3.0[3]),
            u64x4::new(x0.0[4], x1.0[4], x2.0[4], x3.0[4]),
        ])
    }

    pub fn split(&self) -> [FieldElement51; 4] {
        let x = &self.0;
        [
            FieldElement51([
                x[0].extract::<0>(),
                x[1].extract::<0>(),
                x[2].extract::<0>(),
                x[3].extract::<0>(),
                x[4].extract::<0>(),
            ]),
            FieldElement51([
                x[0].extract::<1>(),
                x[1].extract::<1>(),
                x[2].extract::<1>(),
                x[3].extract::<1>(),
                x[4].extract::<1>(),
            ]),
            FieldElement51([
                x[0].extract::<2>(),
                x[1].extract::<2>(),
                x[2].extract::<2>(),
                x[3].extract::<2>(),
                x[4].extract::<2>(),
            ]),
            FieldElement51([
                x[0].extract::<3>(),
                x[1].extract::<3>(),
                x[2].extract::<3>(),
                x[3].extract::<3>(),
                x[4].extract::<3>(),
            ]),
        ]
    }

    #[inline]
    pub fn diff_sum(&self) -> Self {
        let tmp1 = self.shuffle(Shuffle::BADC);
        let tmp2 = self.blend(&self.negate_lazy(), Lanes::AC);
        tmp1 + tmp2
    }

    #[inline]
    pub fn negate_lazy(&self) -> Self {
        let lo = u64x4::splat(36_028_797_018_963_664);
        let hi = u64x4::splat(36_028_797_018_963_952);
        Self([
            lo - self.0[0],
            hi - self.0[1],
            hi - self.0[2],
            hi - self.0[3],
            hi - self.0[4],
        ])
    }

    #[inline]
    pub fn shuffle(&self, control: Shuffle) -> Self {
        Self([
            shuffle_lanes(self.0[0], control),
            shuffle_lanes(self.0[1], control),
            shuffle_lanes(self.0[2], control),
            shuffle_lanes(self.0[3], control),
            shuffle_lanes(self.0[4], control),
        ])
    }

    #[inline]
    pub fn blend(&self, other: &Self, control: Lanes) -> Self {
        Self([
            blend_lanes(self.0[0], other.0[0], control),
            blend_lanes(self.0[1], other.0[1], control),
            blend_lanes(self.0[2], other.0[2], control),
            blend_lanes(self.0[3], other.0[3], control),
            blend_lanes(self.0[4], other.0[4], control),
        ])
    }
}

#[macros::target_feature("avx512ifma,avx512vl")]
impl Add<Self> for F51x4Unreduced {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
            self.0[3] + rhs.0[3],
            self.0[4] + rhs.0[4],
        ])
    }
}

#[derive(Debug, Clone, Copy)]
pub struct F51x4Reduced(pub(crate) [u64x4; 5]);

#[macros::target_feature("avx512ifma,avx512vl")]
impl F51x4Reduced {
    #[inline]
    pub fn shuffle(&self, control: Shuffle) -> Self {
        Self([
            shuffle_lanes(self.0[0], control),
            shuffle_lanes(self.0[1], control),
            shuffle_lanes(self.0[2], control),
            shuffle_lanes(self.0[3], control),
            shuffle_lanes(self.0[4], control),
        ])
    }

    #[inline]
    pub fn blend(&self, other: &Self, control: Lanes) -> Self {
        Self([
            blend_lanes(self.0[0], other.0[0], control),
            blend_lanes(self.0[1], other.0[1], control),
            blend_lanes(self.0[2], other.0[2], control),
            blend_lanes(self.0[3], other.0[3], control),
            blend_lanes(self.0[4], other.0[4], control),
        ])
    }

    #[inline]
    pub fn square(&self) -> F51x4Unreduced {
        unsafe {
            let x = &self.0;
            let mut z0_2 = u64x4::splat(0);
            let mut z1_2 = u64x4::splat(0);
            let mut z2_2 = u64x4::splat(0);
            let mut z3_2 = u64x4::splat(0);
            let mut z4_2 = u64x4::splat(0);
            let mut z5_2 = u64x4::splat(0);
            let mut z6_2 = u64x4::splat(0);
            let mut z7_2 = u64x4::splat(0);
            let mut z9_2 = u64x4::splat(0);
            let mut z2_4 = u64x4::splat(0);
            let mut z3_4 = u64x4::splat(0);
            let mut z4_4 = u64x4::splat(0);
            let mut z5_4 = u64x4::splat(0);
            let mut z6_4 = u64x4::splat(0);
            let mut z7_4 = u64x4::splat(0);
            let mut z8_4 = u64x4::splat(0);
            let mut z0_1 = u64x4::splat(0);
            z0_1 = madd52lo(z0_1, x[0], x[0]);
            let mut z1_1 = u64x4::splat(0);
            z1_2 = madd52lo(z1_2, x[0], x[1]);
            z1_2 = madd52hi(z1_2, x[0], x[0]);
            z2_4 = madd52hi(z2_4, x[0], x[1]);
            let mut z2_1 = z2_4.shl::<2>();
            z2_2 = madd52lo(z2_2, x[0], x[2]);
            z2_1 = madd52lo(z2_1, x[1], x[1]);
            z3_4 = madd52hi(z3_4, x[0], x[2]);
            let mut z3_1 = z3_4.shl::<2>();
            z3_2 = madd52lo(z3_2, x[1], x[2]);
            z3_2 = madd52lo(z3_2, x[0], x[3]);
            z3_2 = madd52hi(z3_2, x[1], x[1]);
            z4_4 = madd52hi(z4_4, x[1], x[2]);
            z4_4 = madd52hi(z4_4, x[0], x[3]);
            let mut z4_1 = z4_4.shl::<2>();
            z4_2 = madd52lo(z4_2, x[1], x[3]);
            z4_2 = madd52lo(z4_2, x[0], x[4]);
            z4_1 = madd52lo(z4_1, x[2], x[2]);
            z5_4 = madd52hi(z5_4, x[1], x[3]);
            z5_4 = madd52hi(z5_4, x[0], x[4]);
            let mut z5_1 = z5_4.shl::<2>();
            z5_2 = madd52lo(z5_2, x[2], x[3]);
            z5_2 = madd52lo(z5_2, x[1], x[4]);
            z5_2 = madd52hi(z5_2, x[2], x[2]);
            z6_4 = madd52hi(z6_4, x[2], x[3]);
            z6_4 = madd52hi(z6_4, x[1], x[4]);
            let mut z6_1 = z6_4.shl::<2>();
            z6_2 = madd52lo(z6_2, x[2], x[4]);
            z6_1 = madd52lo(z6_1, x[3], x[3]);
            z7_4 = madd52hi(z7_4, x[2], x[4]);
            let mut z7_1 = z7_4.shl::<2>();
            z7_2 = madd52lo(z7_2, x[3], x[4]);
            z7_2 = madd52hi(z7_2, x[3], x[3]);
            z8_4 = madd52hi(z8_4, x[3], x[4]);
            let mut z8_1 = z8_4.shl::<2>();
            z8_1 = madd52lo(z8_1, x[4], x[4]);
            let mut z9_1 = u64x4::splat(0);
            z9_2 = madd52hi(z9_2, x[4], x[4]);
            z5_1 += z5_2.shl::<1>();
            z6_1 += z6_2.shl::<1>();
            z7_1 += z7_2.shl::<1>();
            z9_1 += z9_2.shl::<1>();
            let mut t0 = u64x4::splat(0);
            let mut t1 = u64x4::splat(0);
            let r19 = u64x4::splat(19);
            t0 = madd52hi(t0, r19, z9_1);
            t1 = madd52lo(t1, r19, z9_1.shr::<52>());
            z4_2 = madd52lo(z4_2, r19, z8_1.shr::<52>());
            z3_2 = madd52lo(z3_2, r19, z7_1.shr::<52>());
            z2_2 = madd52lo(z2_2, r19, z6_1.shr::<52>());
            z1_2 = madd52lo(z1_2, r19, z5_1.shr::<52>());
            z0_2 = madd52lo(z0_2, r19, t0 + t1);
            z1_2 = madd52hi(z1_2, r19, z5_1);
            z2_2 = madd52hi(z2_2, r19, z6_1);
            z3_2 = madd52hi(z3_2, r19, z7_1);
            z4_2 = madd52hi(z4_2, r19, z8_1);
            z0_1 = madd52lo(z0_1, r19, z5_1);
            z1_1 = madd52lo(z1_1, r19, z6_1);
            z2_1 = madd52lo(z2_1, r19, z7_1);
            z3_1 = madd52lo(z3_1, r19, z8_1);
            z4_1 = madd52lo(z4_1, r19, z9_1);
            F51x4Unreduced([
                z0_1 + z0_2 + z0_2,
                z1_1 + z1_2 + z1_2,
                z2_1 + z2_2 + z2_2,
                z3_1 + z3_2 + z3_2,
                z4_1 + z4_2 + z4_2,
            ])
        }
    }
}

#[macros::target_feature("avx512ifma,avx512vl")]
impl Neg for F51x4Reduced {
    type Output = Self;

    fn neg(self) -> Self {
        F51x4Unreduced::from(self).negate_lazy().into()
    }
}

#[macros::target_feature("avx512ifma,avx512vl")]
impl<'a, 'b> Mul<&'b F51x4Reduced> for &'a F51x4Reduced {
    type Output = F51x4Unreduced;

    #[inline]
    fn mul(self, rhs: &'b F51x4Reduced) -> F51x4Unreduced {
        unsafe {
            let x = &self.0;
            let y = &rhs.0;
            let mut z0_1 = u64x4::splat(0);
            let mut z1_1 = u64x4::splat(0);
            let mut z2_1 = u64x4::splat(0);
            let mut z3_1 = u64x4::splat(0);
            let mut z4_1 = u64x4::splat(0);
            let mut z5_1 = u64x4::splat(0);
            let mut z6_1 = u64x4::splat(0);
            let mut z7_1 = u64x4::splat(0);
            let mut z8_1 = u64x4::splat(0);
            let mut z0_2 = u64x4::splat(0);
            let mut z1_2 = u64x4::splat(0);
            let mut z2_2 = u64x4::splat(0);
            let mut z3_2 = u64x4::splat(0);
            let mut z4_2 = u64x4::splat(0);
            let mut z5_2 = u64x4::splat(0);
            let mut z6_2 = u64x4::splat(0);
            let mut z7_2 = u64x4::splat(0);
            let mut z8_2 = u64x4::splat(0);
            let mut z9_2 = u64x4::splat(0);
            z4_1 = madd52lo(z4_1, x[2], y[2]);
            z5_2 = madd52hi(z5_2, x[2], y[2]);
            z5_1 = madd52lo(z5_1, x[4], y[1]);
            z6_2 = madd52hi(z6_2, x[4], y[1]);
            z6_1 = madd52lo(z6_1, x[4], y[2]);
            z7_2 = madd52hi(z7_2, x[4], y[2]);
            z7_1 = madd52lo(z7_1, x[4], y[3]);
            z8_2 = madd52hi(z8_2, x[4], y[3]);
            z4_1 = madd52lo(z4_1, x[3], y[1]);
            z5_2 = madd52hi(z5_2, x[3], y[1]);
            z5_1 = madd52lo(z5_1, x[3], y[2]);
            z6_2 = madd52hi(z6_2, x[3], y[2]);
            z6_1 = madd52lo(z6_1, x[3], y[3]);
            z7_2 = madd52hi(z7_2, x[3], y[3]);
            z7_1 = madd52lo(z7_1, x[3], y[4]);
            z8_2 = madd52hi(z8_2, x[3], y[4]);
            z8_1 = madd52lo(z8_1, x[4], y[4]);
            z9_2 = madd52hi(z9_2, x[4], y[4]);
            z4_1 = madd52lo(z4_1, x[4], y[0]);
            z5_2 = madd52hi(z5_2, x[4], y[0]);
            z5_1 = madd52lo(z5_1, x[2], y[3]);
            z6_2 = madd52hi(z6_2, x[2], y[3]);
            z6_1 = madd52lo(z6_1, x[2], y[4]);
            z7_2 = madd52hi(z7_2, x[2], y[4]);
            let z8 = z8_1 + z8_2 + z8_2;
            let z9 = z9_2 + z9_2;
            z3_1 = madd52lo(z3_1, x[3], y[0]);
            z4_2 = madd52hi(z4_2, x[3], y[0]);
            z4_1 = madd52lo(z4_1, x[1], y[3]);
            z5_2 = madd52hi(z5_2, x[1], y[3]);
            z5_1 = madd52lo(z5_1, x[1], y[4]);
            z6_2 = madd52hi(z6_2, x[1], y[4]);
            z2_1 = madd52lo(z2_1, x[2], y[0]);
            z3_2 = madd52hi(z3_2, x[2], y[0]);
            let z6 = z6_1 + z6_2 + z6_2;
            let z7 = z7_1 + z7_2 + z7_2;
            z3_1 = madd52lo(z3_1, x[2], y[1]);
            z4_2 = madd52hi(z4_2, x[2], y[1]);
            z4_1 = madd52lo(z4_1, x[0], y[4]);
            z5_2 = madd52hi(z5_2, x[0], y[4]);
            z1_1 = madd52lo(z1_1, x[1], y[0]);
            z2_2 = madd52hi(z2_2, x[1], y[0]);
            z2_1 = madd52lo(z2_1, x[1], y[1]);
            z3_2 = madd52hi(z3_2, x[1], y[1]);
            let z5 = z5_1 + z5_2 + z5_2;
            z3_1 = madd52lo(z3_1, x[1], y[2]);
            z4_2 = madd52hi(z4_2, x[1], y[2]);
            z0_1 = madd52lo(z0_1, x[0], y[0]);
            z1_2 = madd52hi(z1_2, x[0], y[0]);
            z1_1 = madd52lo(z1_1, x[0], y[1]);
            z2_1 = madd52lo(z2_1, x[0], y[2]);
            z2_2 = madd52hi(z2_2, x[0], y[1]);
            z3_2 = madd52hi(z3_2, x[0], y[2]);
            let mut t0 = u64x4::splat(0);
            let mut t1 = u64x4::splat(0);
            let r19 = u64x4::splat(19);
            t0 = madd52hi(t0, r19, z9);
            t1 = madd52lo(t1, r19, z9.shr::<52>());
            z3_1 = madd52lo(z3_1, x[0], y[3]);
            z4_2 = madd52hi(z4_2, x[0], y[3]);
            z1_2 = madd52lo(z1_2, r19, z5.shr::<52>());
            z2_2 = madd52lo(z2_2, r19, z6.shr::<52>());
            z3_2 = madd52lo(z3_2, r19, z7.shr::<52>());
            z0_1 = madd52lo(z0_1, r19, z5);
            z4_1 = madd52lo(z4_1, r19, z9);
            z1_1 = madd52lo(z1_1, r19, z6);
            z0_2 = madd52lo(z0_2, r19, t0 + t1);
            z4_2 = madd52hi(z4_2, r19, z8);
            z2_1 = madd52lo(z2_1, r19, z7);
            z1_2 = madd52hi(z1_2, r19, z5);
            z2_2 = madd52hi(z2_2, r19, z6);
            z3_2 = madd52hi(z3_2, r19, z7);
            z3_1 = madd52lo(z3_1, r19, z8);
            z4_2 = madd52lo(z4_2, r19, z8.shr::<52>());
            F51x4Unreduced([
                z0_1 + z0_2 + z0_2,
                z1_1 + z1_2 + z1_2,
                z2_1 + z2_2 + z2_2,
                z3_1 + z3_2 + z3_2,
                z4_1 + z4_2 + z4_2,
            ])
        }
    }
}

#[macros::target_feature("avx512ifma,avx512vl")]
impl<'a> Mul<(u32, u32, u32, u32)> for &'a F51x4Reduced {
    type Output = F51x4Unreduced;

    #[inline]
    #[allow(clippy::cast_lossless)]
    fn mul(self, rhs: (u32, u32, u32, u32)) -> F51x4Unreduced {
        unsafe {
            let x = &self.0;
            let y = u64x4::new(rhs.0 as u64, rhs.1 as u64, rhs.2 as u64, rhs.3 as u64);
            let r19 = u64x4::splat(19);
            let mut z0_1 = u64x4::splat(0);
            let mut z1_1 = u64x4::splat(0);
            let mut z2_1 = u64x4::splat(0);
            let mut z3_1 = u64x4::splat(0);
            let mut z4_1 = u64x4::splat(0);
            let mut z1_2 = u64x4::splat(0);
            let mut z2_2 = u64x4::splat(0);
            let mut z3_2 = u64x4::splat(0);
            let mut z4_2 = u64x4::splat(0);
            let mut z5_2 = u64x4::splat(0);
            z4_2 = madd52hi(z4_2, y, x[3]);
            z5_2 = madd52hi(z5_2, y, x[4]);
            z4_1 = madd52lo(z4_1, y, x[4]);
            z0_1 = madd52lo(z0_1, y, x[0]);
            z3_1 = madd52lo(z3_1, y, x[3]);
            z2_1 = madd52lo(z2_1, y, x[2]);
            z1_1 = madd52lo(z1_1, y, x[1]);
            z3_2 = madd52hi(z3_2, y, x[2]);
            z2_2 = madd52hi(z2_2, y, x[1]);
            z1_2 = madd52hi(z1_2, y, x[0]);
            z0_1 = madd52lo(z0_1, z5_2 + z5_2, r19);
            F51x4Unreduced([
                z0_1,
                z1_1 + z1_2 + z1_2,
                z2_1 + z2_2 + z2_2,
                z3_1 + z3_2 + z3_2,
                z4_1 + z4_2 + z4_2,
            ])
        }
    }
}

#[macros::target_feature("avx512ifma,avx512vl")]
impl From<F51x4Reduced> for F51x4Unreduced {
    #[inline]
    fn from(x: F51x4Reduced) -> Self {
        Self(x.0)
    }
}

#[macros::target_feature("avx512ifma,avx512vl")]
impl From<F51x4Unreduced> for F51x4Reduced {
    #[inline]
    fn from(x: F51x4Unreduced) -> Self {
        let mask = u64x4::splat((1 << 51) - 1);
        let r19 = u64x4::splat(19);
        let c0 = x.0[0].shr::<51>();
        let c1 = x.0[1].shr::<51>();
        let c2 = x.0[2].shr::<51>();
        let c3 = x.0[3].shr::<51>();
        let c4 = x.0[4].shr::<51>();
        unsafe {
            Self([
                madd52lo(x.0[0] & mask, c4, r19),
                (x.0[1] & mask) + c0,
                (x.0[2] & mask) + c1,
                (x.0[3] & mask) + c2,
                (x.0[4] & mask) + c3,
            ])
        }
    }
}
