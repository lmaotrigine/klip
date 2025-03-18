use crate::backends::{
    serial::u64::field::FieldElement51,
    vector::{
        avx2::consts::{P_TIMES_16_HI, P_TIMES_16_LO, P_TIMES_2_HI, P_TIMES_2_LO},
        simd::{u32x8, u64x4},
    },
};
use core::ops::{Add, Mul, Neg};
use crypto_common::constant_time::{Choice, ConditionallySelectable};

const A_LANES: u8 = 0b0000_0101;
const B_LANES: u8 = 0b0000_1010;
const C_LANES: u8 = 0b0101_0000;
const D_LANES: u8 = 0b1010_0000;

#[macros::target_feature("avx2")]
#[inline(always)]
fn unpack_pair(src: u32x8) -> (u32x8, u32x8) {
    let a;
    let b;
    let zero = u32x8::splat(0);
    unsafe {
        use core::arch::x86_64::{_mm256_unpackhi_epi32, _mm256_unpacklo_epi32};
        a = _mm256_unpacklo_epi32(src.into(), zero.into()).into();
        b = _mm256_unpackhi_epi32(src.into(), zero.into()).into();
    }
    (a, b)
}

#[macros::target_feature("avx2")]
#[inline(always)]
fn repack_pair(x: u32x8, y: u32x8) -> u32x8 {
    unsafe {
        use core::arch::x86_64::{_mm256_blend_epi32, _mm256_shuffle_epi32};
        let x_shuffled = _mm256_shuffle_epi32(x.into(), 0b11_01_10_00);
        let y_shuffled = _mm256_shuffle_epi32(y.into(), 0b10_00_11_01);
        _mm256_blend_epi32(x_shuffled, y_shuffled, 0b11_00_11_00).into()
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub enum Lanes {
    C,
    D,
    AB,
    AC,
    CD,
    AD,
    BC,
    ABCD,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub enum Shuffle {
    AAAA,
    BBBB,
    CACA,
    DBBD,
    ADDA,
    CBCB,
    ABAB,
    BADC,
    BACD,
    ABDC,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy)]
pub struct FieldElement2625x4(pub(crate) [u32x8; 5]);

#[macros::target_feature("avx2")]
impl FieldElement2625x4 {
    pub const ZERO: Self = Self([u32x8::splat_const::<0>(); 5]);

    #[allow(clippy::cast_lossless)]
    pub fn split(&self) -> [FieldElement51; 4] {
        let mut out = [FieldElement51::ZERO; 4];
        for i in 0..5 {
            let a_2i = self.0[i].extract::<0>() as u64;
            let b_2i = self.0[i].extract::<1>() as u64;
            let a_2i_1 = self.0[i].extract::<2>() as u64;
            let b_2i_1 = self.0[i].extract::<3>() as u64;
            let c_2i = self.0[i].extract::<4>() as u64;
            let d_2i = self.0[i].extract::<5>() as u64;
            let c_2i_1 = self.0[i].extract::<6>() as u64;
            let d_2i_1 = self.0[i].extract::<7>() as u64;
            out[0].0[i] = a_2i + (a_2i_1 << 26);
            out[1].0[i] = b_2i + (b_2i_1 << 26);
            out[2].0[i] = c_2i + (c_2i_1 << 26);
            out[3].0[i] = d_2i + (d_2i_1 << 26);
        }
        out
    }

    #[inline]
    pub fn shuffle(&self, control: Shuffle) -> Self {
        #[inline(always)]
        fn shuffle_lanes(x: u32x8, control: Shuffle) -> u32x8 {
            unsafe {
                use core::arch::x86_64::_mm256_permutevar8x32_epi32;
                let c = match control {
                    Shuffle::AAAA => u32x8::new(0, 0, 2, 2, 0, 0, 2, 2),
                    Shuffle::BBBB => u32x8::new(1, 1, 3, 3, 1, 1, 3, 3),
                    Shuffle::CACA => u32x8::new(4, 0, 6, 2, 4, 0, 6, 2),
                    Shuffle::DBBD => u32x8::new(5, 1, 7, 3, 1, 5, 3, 7),
                    Shuffle::ADDA => u32x8::new(0, 5, 2, 7, 5, 0, 7, 2),
                    Shuffle::CBCB => u32x8::new(4, 1, 6, 3, 4, 1, 6, 3),
                    Shuffle::ABAB => u32x8::new(0, 1, 2, 3, 0, 1, 2, 3),
                    Shuffle::BADC => u32x8::new(1, 0, 3, 2, 5, 4, 7, 6),
                    Shuffle::BACD => u32x8::new(1, 0, 3, 2, 4, 5, 6, 7),
                    Shuffle::ABDC => u32x8::new(0, 1, 2, 3, 5, 4, 7, 6),
                };
                _mm256_permutevar8x32_epi32(x.into(), c.into()).into()
            }
        }
        Self([
            shuffle_lanes(self.0[0], control),
            shuffle_lanes(self.0[1], control),
            shuffle_lanes(self.0[2], control),
            shuffle_lanes(self.0[3], control),
            shuffle_lanes(self.0[4], control),
        ])
    }

    #[inline]
    pub fn blend(&self, other: Self, control: Lanes) -> Self {
        #[inline(always)]
        fn blend_lanes(x: u32x8, y: u32x8, control: Lanes) -> u32x8 {
            unsafe {
                use core::arch::x86_64::_mm256_blend_epi32;
                match control {
                    Lanes::C => _mm256_blend_epi32(x.into(), y.into(), C_LANES as i32).into(),
                    Lanes::D => _mm256_blend_epi32(x.into(), y.into(), D_LANES as i32).into(),
                    Lanes::AD => {
                        _mm256_blend_epi32(x.into(), y.into(), (A_LANES | D_LANES) as i32).into()
                    }
                    Lanes::AB => {
                        _mm256_blend_epi32(x.into(), y.into(), (A_LANES | B_LANES) as i32).into()
                    }
                    Lanes::AC => {
                        _mm256_blend_epi32(x.into(), y.into(), (A_LANES | C_LANES) as i32).into()
                    }
                    Lanes::CD => {
                        _mm256_blend_epi32(x.into(), y.into(), (C_LANES | D_LANES) as i32).into()
                    }
                    Lanes::BC => {
                        _mm256_blend_epi32(x.into(), y.into(), (B_LANES | C_LANES) as i32).into()
                    }
                    Lanes::ABCD => _mm256_blend_epi32(
                        x.into(),
                        y.into(),
                        (A_LANES | B_LANES | C_LANES | D_LANES) as i32,
                    )
                    .into(),
                }
            }
        }
        Self([
            blend_lanes(self.0[0], other.0[0], control),
            blend_lanes(self.0[1], other.0[1], control),
            blend_lanes(self.0[2], other.0[2], control),
            blend_lanes(self.0[3], other.0[3], control),
            blend_lanes(self.0[4], other.0[4], control),
        ])
    }

    pub fn new(
        x0: &FieldElement51,
        x1: &FieldElement51,
        x2: &FieldElement51,
        x3: &FieldElement51,
    ) -> Self {
        let mut buf = [u32x8::splat(0); 5];
        let low_26_bits = (1 << 26) - 1;
        #[allow(clippy::needless_range_loop, clippy::cast_possible_truncation)]
        for i in 0..5 {
            let a_2i = (x0.0[i] & low_26_bits) as u32;
            let a_2i_1 = (x0.0[i] >> 26) as u32;
            let b_2i = (x1.0[i] & low_26_bits) as u32;
            let b_2i_1 = (x1.0[i] >> 26) as u32;
            let c_2i = (x2.0[i] & low_26_bits) as u32;
            let c_2i_1 = (x2.0[i] >> 26) as u32;
            let d_2i = (x3.0[i] & low_26_bits) as u32;
            let d_2i_1 = (x3.0[i] >> 26) as u32;
            buf[i] = u32x8::new(a_2i, b_2i, a_2i_1, b_2i_1, c_2i, d_2i, c_2i_1, d_2i_1);
        }
        Self(buf).reduce()
    }

    #[inline]
    pub fn negate_lazy(&self) -> Self {
        Self([
            P_TIMES_2_LO - self.0[0],
            P_TIMES_2_HI - self.0[1],
            P_TIMES_2_HI - self.0[2],
            P_TIMES_2_HI - self.0[3],
            P_TIMES_2_HI - self.0[4],
        ])
    }

    #[inline]
    pub fn diff_sum(&self) -> Self {
        let tmp1 = self.shuffle(Shuffle::BADC);
        let tmp2 = self.blend(self.negate_lazy(), Lanes::AC);
        tmp1 + tmp2
    }

    #[inline]
    pub fn reduce(&self) -> Self {
        use core::arch::x86_64::{
            _mm256_blend_epi32, _mm256_mul_epu32, _mm256_shuffle_epi32, _mm256_srlv_epi32,
        };
        let shifts = u32x8::new(26, 26, 25, 25, 26, 26, 25, 25);
        let masks = u32x8::new(
            (1 << 26) - 1,
            (1 << 26) - 1,
            (1 << 25) - 1,
            (1 << 25) - 1,
            (1 << 26) - 1,
            (1 << 26) - 1,
            (1 << 25) - 1,
            (1 << 25) - 1,
        );
        let rotated_carryout = |v: u32x8| -> u32x8 {
            unsafe {
                let c = _mm256_srlv_epi32(v.into(), shifts.into());
                _mm256_shuffle_epi32(c, 0b01_00_11_10).into()
            }
        };
        let combine = |v_lo: u32x8, v_hi: u32x8| -> u32x8 {
            unsafe { _mm256_blend_epi32(v_lo.into(), v_hi.into(), 0b11_00_11_00).into() }
        };
        let mut v = self.0;
        let c10 = rotated_carryout(v[0]);
        v[0] = (v[0] & masks) + combine(u32x8::splat(0), c10);
        let c32 = rotated_carryout(v[1]);
        v[1] = (v[1] & masks) + combine(c10, c32);
        let c54 = rotated_carryout(v[2]);
        v[2] = (v[2] & masks) + combine(c32, c54);
        let c76 = rotated_carryout(v[3]);
        v[3] = (v[3] & masks) + combine(c54, c76);
        let c98 = rotated_carryout(v[4]);
        v[4] = (v[4] & masks) + combine(c76, c98);
        let c9_19 = unsafe {
            let c9_spread = _mm256_shuffle_epi32(c98.into(), 0b11_01_10_00);
            let c9_19_spread = _mm256_mul_epu32(c9_spread, u64x4::splat(19).into());
            _mm256_shuffle_epi32(c9_19_spread, 0b11_01_10_00).into()
        };
        v[0] += c9_19;
        Self(v)
    }

    #[inline]
    fn reduce64(mut z: [u64x4; 10]) -> Self {
        let low_25_bits = u64x4::splat((1 << 25) - 1);
        let low_26_bits = u64x4::splat((1 << 26) - 1);
        let carry = |z: &mut [u64x4; 10], i: usize| {
            debug_assert!(i < 9);
            if i % 2 == 0 {
                z[i + 1] += z[i].shr::<26>();
                z[i] &= low_26_bits;
            } else {
                z[i + 1] += z[i].shr::<25>();
                z[i] &= low_25_bits;
            }
        };
        carry(&mut z, 0);
        carry(&mut z, 4);
        carry(&mut z, 1);
        carry(&mut z, 5);
        carry(&mut z, 2);
        carry(&mut z, 6);
        carry(&mut z, 3);
        carry(&mut z, 7);
        carry(&mut z, 4);
        carry(&mut z, 8);
        let c = z[9].shr::<25>();
        z[9] &= low_25_bits;
        let mut c0 = c & low_26_bits;
        let mut c1 = c.shr::<26>();
        let x19 = u64x4::splat(19);
        c0 = u32x8::from(c0).mul32(u32x8::from(x19));
        c1 = u32x8::from(c1).mul32(u32x8::from(x19));
        z[0] += c0;
        z[1] += c1;
        carry(&mut z, 0);
        Self([
            repack_pair(z[0].into(), z[1].into()),
            repack_pair(z[2].into(), z[3].into()),
            repack_pair(z[4].into(), z[5].into()),
            repack_pair(z[6].into(), z[7].into()),
            repack_pair(z[8].into(), z[9].into()),
        ])
    }

    pub fn square_and_negate_d(&self) -> Self {
        #[inline(always)]
        fn m(x: u32x8, y: u32x8) -> u64x4 {
            x.mul32(y)
        }
        #[inline(always)]
        fn m_lo(x: u32x8, y: u32x8) -> u32x8 {
            x.mul32(y).into()
        }
        let v19 = u32x8::new(19, 0, 19, 0, 19, 0, 19, 0);
        let (x0, x1) = unpack_pair(self.0[0]);
        let (x2, x3) = unpack_pair(self.0[1]);
        let (x4, x5) = unpack_pair(self.0[2]);
        let (x6, x7) = unpack_pair(self.0[3]);
        let (x8, x9) = unpack_pair(self.0[4]);
        let x0_2 = x0.shl::<1>();
        let x1_2 = x1.shl::<1>();
        let x2_2 = x2.shl::<1>();
        let x3_2 = x3.shl::<1>();
        let x4_2 = x4.shl::<1>();
        let x5_2 = x5.shl::<1>();
        let x6_2 = x6.shl::<1>();
        let x7_2 = x7.shl::<1>();
        let x5_19 = m_lo(v19, x5);
        let x6_19 = m_lo(v19, x6);
        let x7_19 = m_lo(v19, x7);
        let x8_19 = m_lo(v19, x8);
        let x9_19 = m_lo(v19, x9);
        let mut z0 = m(x0, x0)
            + m(x2_2, x8_19)
            + m(x4_2, x6_19)
            + ((m(x1_2, x9_19) + m(x3_2, x7_19) + m(x5, x5_19)).shl::<1>());
        let mut z1 = m(x0_2, x1)
            + m(x3_2, x8_19)
            + m(x5_2, x6_19)
            + ((m(x2, x9_19) + m(x4, x7_19)).shl::<1>());
        let mut z2 = m(x0_2, x2)
            + m(x1_2, x1)
            + m(x4_2, x8_19)
            + m(x6, x6_19)
            + ((m(x3_2, x9_19) + m(x5_2, x7_19)).shl::<1>());
        let mut z3 =
            m(x0_2, x3) + m(x1_2, x2) + m(x5_2, x8_19) + ((m(x4, x9_19) + m(x6, x7_19)).shl::<1>());
        let mut z4 = m(x0_2, x4)
            + m(x1_2, x3_2)
            + m(x2, x2)
            + m(x6_2, x8_19)
            + ((m(x5_2, x9_19) + m(x7, x7_19)).shl::<1>());
        let mut z5 =
            m(x0_2, x5) + m(x1_2, x4) + m(x2_2, x3) + m(x7_2, x8_19) + ((m(x6, x9_19)).shl::<1>());
        let mut z6 = m(x0_2, x6)
            + m(x1_2, x5_2)
            + m(x2_2, x4)
            + m(x3_2, x3)
            + m(x8, x8_19)
            + ((m(x7_2, x9_19)).shl::<1>());
        let mut z7 =
            m(x0_2, x7) + m(x1_2, x6) + m(x2_2, x5) + m(x3_2, x4) + ((m(x8, x9_19)).shl::<1>());
        let mut z8 = m(x0_2, x8)
            + m(x1_2, x7_2)
            + m(x2_2, x6)
            + m(x3_2, x5_2)
            + m(x4, x4)
            + ((m(x9, x9_19)).shl::<1>());
        let mut z9 = m(x0_2, x9) + m(x1_2, x8) + m(x2_2, x7) + m(x3_2, x6) + m(x4_2, x5);
        let low_p37 = u64x4::splat(0x03ff_ffed << 37);
        let even_p37 = u64x4::splat(0x03ff_ffff << 37);
        let odd_p37 = u64x4::splat(0x01ff_ffff << 37);
        let negate_d = |x: u64x4, p: u64x4| -> u64x4 {
            unsafe {
                use core::arch::x86_64::_mm256_blend_epi32;
                _mm256_blend_epi32(x.into(), (p - x).into(), 0b11_00_00_00).into()
            }
        };
        z0 = negate_d(z0, low_p37);
        z1 = negate_d(z1, odd_p37);
        z2 = negate_d(z2, even_p37);
        z3 = negate_d(z3, odd_p37);
        z4 = negate_d(z4, even_p37);
        z5 = negate_d(z5, odd_p37);
        z6 = negate_d(z6, even_p37);
        z7 = negate_d(z7, odd_p37);
        z8 = negate_d(z8, even_p37);
        z9 = negate_d(z9, odd_p37);
        Self::reduce64([z0, z1, z2, z3, z4, z5, z6, z7, z8, z9])
    }
}

#[macros::target_feature("avx2")]
#[allow(clippy::cast_sign_loss, clippy::cast_lossless)]
impl ConditionallySelectable for FieldElement2625x4 {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let mask = (-(choice.to_u8() as i32)) as u32;
        let mask_vec = u32x8::splat(mask);
        Self([
            a.0[0] ^ (mask_vec & (a.0[0] ^ b.0[0])),
            a.0[1] ^ (mask_vec & (a.0[1] ^ b.0[1])),
            a.0[2] ^ (mask_vec & (a.0[2] ^ b.0[2])),
            a.0[3] ^ (mask_vec & (a.0[3] ^ b.0[3])),
            a.0[4] ^ (mask_vec & (a.0[4] ^ b.0[4])),
        ])
    }

    fn conditional_assign(&mut self, other: &Self, choice: Choice) {
        let mask = (-(choice.to_u8() as i32)) as u32;
        let mask_vec = u32x8::splat(mask);
        self.0[0] ^= mask_vec & (self.0[0] ^ other.0[0]);
        self.0[1] ^= mask_vec & (self.0[1] ^ other.0[1]);
        self.0[2] ^= mask_vec & (self.0[2] ^ other.0[2]);
        self.0[3] ^= mask_vec & (self.0[3] ^ other.0[3]);
        self.0[4] ^= mask_vec & (self.0[4] ^ other.0[4]);
    }
}

#[macros::target_feature("avx2")]
impl Neg for FieldElement2625x4 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self([
            P_TIMES_16_LO - self.0[0],
            P_TIMES_16_HI - self.0[1],
            P_TIMES_16_HI - self.0[2],
            P_TIMES_16_HI - self.0[3],
            P_TIMES_16_HI - self.0[4],
        ])
        .reduce()
    }
}

#[macros::target_feature("avx2")]
impl Add<Self> for FieldElement2625x4 {
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

#[macros::target_feature("avx2")]
impl Mul<(u32, u32, u32, u32)> for FieldElement2625x4 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: (u32, u32, u32, u32)) -> Self {
        let consts = u32x8::new(rhs.0, 0, rhs.1, 0, rhs.2, 0, rhs.3, 0);
        let (b0, b1) = unpack_pair(self.0[0]);
        let (b2, b3) = unpack_pair(self.0[1]);
        let (b4, b5) = unpack_pair(self.0[2]);
        let (b6, b7) = unpack_pair(self.0[3]);
        let (b8, b9) = unpack_pair(self.0[4]);
        Self::reduce64([
            b0.mul32(consts),
            b1.mul32(consts),
            b2.mul32(consts),
            b3.mul32(consts),
            b4.mul32(consts),
            b5.mul32(consts),
            b6.mul32(consts),
            b7.mul32(consts),
            b8.mul32(consts),
            b9.mul32(consts),
        ])
    }
}

#[macros::target_feature("avx2")]
impl Mul<Self> for &FieldElement2625x4 {
    type Output = FieldElement2625x4;

    #[inline]
    fn mul(self, rhs: Self) -> FieldElement2625x4 {
        #[inline(always)]
        fn m(x: u32x8, y: u32x8) -> u64x4 {
            x.mul32(y)
        }
        #[inline(always)]
        fn m_lo(x: u32x8, y: u32x8) -> u32x8 {
            x.mul32(y).into()
        }
        let (x0, x1) = unpack_pair(self.0[0]);
        let (x2, x3) = unpack_pair(self.0[1]);
        let (x4, x5) = unpack_pair(self.0[2]);
        let (x6, x7) = unpack_pair(self.0[3]);
        let (x8, x9) = unpack_pair(self.0[4]);
        let (y0, y1) = unpack_pair(rhs.0[0]);
        let (y2, y3) = unpack_pair(rhs.0[1]);
        let (y4, y5) = unpack_pair(rhs.0[2]);
        let (y6, y7) = unpack_pair(rhs.0[3]);
        let (y8, y9) = unpack_pair(rhs.0[4]);
        let v19 = u32x8::new(19, 0, 19, 0, 19, 0, 19, 0);
        let y1_19 = m_lo(v19, y1);
        let y2_19 = m_lo(v19, y2);
        let y3_19 = m_lo(v19, y3);
        let y4_19 = m_lo(v19, y4);
        let y5_19 = m_lo(v19, y5);
        let y6_19 = m_lo(v19, y6);
        let y7_19 = m_lo(v19, y7);
        let y8_19 = m_lo(v19, y8);
        let y9_19 = m_lo(v19, y9);
        let x1_2 = x1 + x1;
        let x3_2 = x3 + x3;
        let x5_2 = x5 + x5;
        let x7_2 = x7 + x7;
        let x9_2 = x9 + x9;
        let z0 = m(x0, y0)
            + m(x1_2, y9_19)
            + m(x2, y8_19)
            + m(x3_2, y7_19)
            + m(x4, y6_19)
            + m(x5_2, y5_19)
            + m(x6, y4_19)
            + m(x7_2, y3_19)
            + m(x8, y2_19)
            + m(x9_2, y1_19);
        let z1 = m(x0, y1)
            + m(x1, y0)
            + m(x2, y9_19)
            + m(x3, y8_19)
            + m(x4, y7_19)
            + m(x5, y6_19)
            + m(x6, y5_19)
            + m(x7, y4_19)
            + m(x8, y3_19)
            + m(x9, y2_19);
        let z2 = m(x0, y2)
            + m(x1_2, y1)
            + m(x2, y0)
            + m(x3_2, y9_19)
            + m(x4, y8_19)
            + m(x5_2, y7_19)
            + m(x6, y6_19)
            + m(x7_2, y5_19)
            + m(x8, y4_19)
            + m(x9_2, y3_19);
        let z3 = m(x0, y3)
            + m(x1, y2)
            + m(x2, y1)
            + m(x3, y0)
            + m(x4, y9_19)
            + m(x5, y8_19)
            + m(x6, y7_19)
            + m(x7, y6_19)
            + m(x8, y5_19)
            + m(x9, y4_19);
        let z4 = m(x0, y4)
            + m(x1_2, y3)
            + m(x2, y2)
            + m(x3_2, y1)
            + m(x4, y0)
            + m(x5_2, y9_19)
            + m(x6, y8_19)
            + m(x7_2, y7_19)
            + m(x8, y6_19)
            + m(x9_2, y5_19);
        let z5 = m(x0, y5)
            + m(x1, y4)
            + m(x2, y3)
            + m(x3, y2)
            + m(x4, y1)
            + m(x5, y0)
            + m(x6, y9_19)
            + m(x7, y8_19)
            + m(x8, y7_19)
            + m(x9, y6_19);
        let z6 = m(x0, y6)
            + m(x1_2, y5)
            + m(x2, y4)
            + m(x3_2, y3)
            + m(x4, y2)
            + m(x5_2, y1)
            + m(x6, y0)
            + m(x7_2, y9_19)
            + m(x8, y8_19)
            + m(x9_2, y7_19);
        let z7 = m(x0, y7)
            + m(x1, y6)
            + m(x2, y5)
            + m(x3, y4)
            + m(x4, y3)
            + m(x5, y2)
            + m(x6, y1)
            + m(x7, y0)
            + m(x8, y9_19)
            + m(x9, y8_19);
        let z8 = m(x0, y8)
            + m(x1_2, y7)
            + m(x2, y6)
            + m(x3_2, y5)
            + m(x4, y4)
            + m(x5_2, y3)
            + m(x6, y2)
            + m(x7_2, y1)
            + m(x8, y0)
            + m(x9_2, y9_19);
        let z9 = m(x0, y9)
            + m(x1, y8)
            + m(x2, y7)
            + m(x3, y6)
            + m(x4, y5)
            + m(x5, y4)
            + m(x6, y3)
            + m(x7, y2)
            + m(x8, y1)
            + m(x9, y0);
        FieldElement2625x4::reduce64([z0, z1, z2, z3, z4, z5, z6, z7, z8, z9])
    }
}
