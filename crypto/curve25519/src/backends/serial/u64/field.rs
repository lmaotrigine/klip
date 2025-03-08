use core::{
    fmt::Debug,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};
use crypto_common::{
    constant_time::{Choice, ConditionallySelectable},
    erase::Erase,
};

#[derive(Clone, Copy)]
pub struct FieldElement51(pub(crate) [u64; 5]);

impl Debug for FieldElement51 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FieldElement51({:?})", &self.0[..])
    }
}

impl Erase for FieldElement51 {
    fn erase(&mut self) {
        self.0.erase();
    }
}

impl FieldElement51 {
    pub const ONE: Self = Self::from_limbs([1, 0, 0, 0, 0]);
    pub const ZERO: Self = Self::from_limbs([0, 0, 0, 0, 0]);

    pub(crate) const fn from_limbs(limbs: [u64; 5]) -> Self {
        Self(limbs)
    }

    pub const fn negate(&mut self) {
        let neg = Self::reduce([
            36_028_797_018_963_664 - self.0[0],
            36_028_797_018_963_952 - self.0[1],
            36_028_797_018_963_952 - self.0[2],
            36_028_797_018_963_952 - self.0[3],
            36_028_797_018_963_952 - self.0[4],
        ]);
        self.0 = neg.0;
    }

    #[inline(always)]
    const fn reduce(mut limbs: [u64; 5]) -> Self {
        const LOW_51_BITS: u64 = (1 << 51) - 1;
        let c0 = limbs[0] >> 51;
        let c1 = limbs[1] >> 51;
        let c2 = limbs[2] >> 51;
        let c3 = limbs[3] >> 51;
        let c4 = limbs[4] >> 51;
        limbs[0] &= LOW_51_BITS;
        limbs[1] &= LOW_51_BITS;
        limbs[2] &= LOW_51_BITS;
        limbs[3] &= LOW_51_BITS;
        limbs[4] &= LOW_51_BITS;
        limbs[0] += c4 * 19;
        limbs[1] += c0;
        limbs[2] += c1;
        limbs[3] += c2;
        limbs[4] += c3;
        Self(limbs)
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        let load8 = |input: &[u8]| -> u64 {
            (input[0] as u64)
                | ((input[1] as u64) << 8)
                | ((input[2] as u64) << 16)
                | ((input[3] as u64) << 24)
                | ((input[4] as u64) << 32)
                | ((input[5] as u64) << 40)
                | ((input[6] as u64) << 48)
                | ((input[7] as u64) << 56)
        };
        let low_51_bits = (1 << 51) - 1;
        Self([
            load8(&bytes[0..]) & low_51_bits,
            (load8(&bytes[6..]) >> 3) & low_51_bits,
            (load8(&bytes[12..]) >> 6) & low_51_bits,
            (load8(&bytes[19..]) >> 1) & low_51_bits,
            (load8(&bytes[24..]) >> 12) & low_51_bits,
        ])
    }

    pub fn as_bytes(&self) -> [u8; 32] {
        let mut limbs = Self::reduce(self.0).0;
        let mut q = (limbs[0] + 19) >> 51;
        q = (limbs[1] + q) >> 51;
        q = (limbs[2] + q) >> 51;
        q = (limbs[3] + q) >> 51;
        q = (limbs[4] + q) >> 51;
        limbs[0] += 19 * q;
        let low_51_bits = (1 << 51) - 1;
        limbs[1] += limbs[0] >> 51;
        limbs[0] &= low_51_bits;
        limbs[2] += limbs[1] >> 51;
        limbs[1] &= low_51_bits;
        limbs[3] += limbs[2] >> 51;
        limbs[2] &= low_51_bits;
        limbs[4] += limbs[3] >> 51;
        limbs[3] &= low_51_bits;
        limbs[4] &= low_51_bits;
        let mut s = [0; 32];
        s[0] = limbs[0] as u8;
        s[1] = (limbs[0] >> 8) as u8;
        s[2] = (limbs[0] >> 16) as u8;
        s[3] = (limbs[0] >> 24) as u8;
        s[4] = (limbs[0] >> 32) as u8;
        s[5] = (limbs[0] >> 40) as u8;
        s[6] = ((limbs[0] >> 48) | (limbs[1] << 3)) as u8;
        s[7] = (limbs[1] >> 5) as u8;
        s[8] = (limbs[1] >> 13) as u8;
        s[9] = (limbs[1] >> 21) as u8;
        s[10] = (limbs[1] >> 29) as u8;
        s[11] = (limbs[1] >> 37) as u8;
        s[12] = ((limbs[1] >> 45) | (limbs[2] << 6)) as u8;
        s[13] = (limbs[2] >> 2) as u8;
        s[14] = (limbs[2] >> 10) as u8;
        s[15] = (limbs[2] >> 18) as u8;
        s[16] = (limbs[2] >> 26) as u8;
        s[17] = (limbs[2] >> 34) as u8;
        s[18] = (limbs[2] >> 42) as u8;
        s[19] = ((limbs[2] >> 50) | (limbs[3] << 1)) as u8;
        s[20] = (limbs[3] >> 7) as u8;
        s[21] = (limbs[3] >> 15) as u8;
        s[22] = (limbs[3] >> 23) as u8;
        s[23] = (limbs[3] >> 31) as u8;
        s[24] = (limbs[3] >> 39) as u8;
        s[25] = ((limbs[3] >> 47) | (limbs[4] << 4)) as u8;
        s[26] = (limbs[4] >> 4) as u8;
        s[27] = (limbs[4] >> 12) as u8;
        s[28] = (limbs[4] >> 20) as u8;
        s[29] = (limbs[4] >> 28) as u8;
        s[30] = (limbs[4] >> 36) as u8;
        s[31] = (limbs[4] >> 44) as u8;
        debug_assert!((s[31] & 0b1000_0000) == 0);
        s
    }

    pub fn pow2k(&self, mut k: u32) -> Self {
        #[inline(always)]
        const fn m(x: u64, y: u64) -> u128 {
            (x as u128) * (y as u128)
        }
        const LOW_51_BITS: u64 = (1 << 51) - 1;
        debug_assert!(k > 0);
        let mut a = self.0;
        loop {
            let a3_19 = 19 * a[3];
            let a4_19 = 19 * a[4];
            let c0 = m(a[0], a[0]) + 2 * (m(a[1], a4_19) + m(a[2], a3_19));
            let mut c1 = m(a[3], a3_19) + 2 * (m(a[0], a[1]) + m(a[2], a4_19));
            let mut c2 = m(a[1], a[1]) + 2 * (m(a[0], a[2]) + m(a[4], a3_19));
            let mut c3 = m(a[4], a4_19) + 2 * (m(a[0], a[3]) + m(a[1], a[2]));
            let mut c4 = m(a[2], a[2]) + 2 * (m(a[0], a[4]) + m(a[1], a[3]));
            debug_assert!(a[0] < (1 << 54));
            debug_assert!(a[1] < (1 << 54));
            debug_assert!(a[2] < (1 << 54));
            debug_assert!(a[3] < (1 << 54));
            debug_assert!(a[4] < (1 << 54));
            c1 += ((c0 >> 51) as u64) as u128;
            a[0] = (c0 as u64) & LOW_51_BITS;
            c2 += ((c1 >> 51) as u64) as u128;
            a[1] = (c1 as u64) & LOW_51_BITS;
            c3 += ((c2 >> 51) as u64) as u128;
            a[2] = (c2 as u64) & LOW_51_BITS;
            c4 += ((c3 >> 51) as u64) as u128;
            a[3] = (c3 as u64) & LOW_51_BITS;
            let carry = (c4 >> 51) as u64;
            a[4] = (c4 as u64) & LOW_51_BITS;
            a[0] += carry * 19;
            a[1] += a[0] >> 51;
            a[0] &= LOW_51_BITS;
            k -= 1;
            if k == 0 {
                break;
            }
        }
        Self(a)
    }

    pub fn square(&self) -> Self {
        self.pow2k(1)
    }

    pub fn square2(&self) -> Self {
        let mut square = self.pow2k(1);
        for i in 0..5 {
            square.0[i] *= 2;
        }
        square
    }
}

impl<'a> AddAssign<&'a Self> for FieldElement51 {
    fn add_assign(&mut self, rhs: &'a Self) {
        for i in 0..5 {
            self.0[i] += rhs.0[i];
        }
    }
}

impl<'b> Add<&'b FieldElement51> for &FieldElement51 {
    type Output = FieldElement51;

    fn add(self, rhs: &'b FieldElement51) -> FieldElement51 {
        let mut sum = *self;
        sum += rhs;
        sum
    }
}

impl<'a> SubAssign<&'a Self> for FieldElement51 {
    fn sub_assign(&mut self, rhs: &'a Self) {
        let result = &*self - rhs;
        self.0 = result.0;
    }
}

impl<'b> Sub<&'b FieldElement51> for &FieldElement51 {
    type Output = FieldElement51;

    fn sub(self, rhs: &'b FieldElement51) -> FieldElement51 {
        FieldElement51::reduce([
            (self.0[0] + 36_028_797_018_963_664) - rhs.0[0],
            (self.0[1] + 36_028_797_018_963_952) - rhs.0[1],
            (self.0[2] + 36_028_797_018_963_952) - rhs.0[2],
            (self.0[3] + 36_028_797_018_963_952) - rhs.0[3],
            (self.0[4] + 36_028_797_018_963_952) - rhs.0[4],
        ])
    }
}

impl<'a> MulAssign<&'a Self> for FieldElement51 {
    fn mul_assign(&mut self, rhs: &'a Self) {
        let result = &*self * rhs;
        self.0 = result.0;
    }
}

impl<'b> Mul<&'b FieldElement51> for &FieldElement51 {
    type Output = FieldElement51;

    fn mul(self, rhs: &'b FieldElement51) -> FieldElement51 {
        const LOW_51_BITS: u64 = (1 << 51) - 1;
        #[inline(always)]
        const fn m(x: u64, y: u64) -> u128 {
            (x as u128) * (y as u128)
        }
        let a = &self.0;
        let b = &rhs.0;
        let b1_19 = b[1] * 19;
        let b2_19 = b[2] * 19;
        let b3_19 = b[3] * 19;
        let b4_19 = b[4] * 19;
        let c0 = m(a[0], b[0]) + m(a[4], b1_19) + m(a[3], b2_19) + m(a[2], b3_19) + m(a[1], b4_19);
        let mut c1 =
            m(a[1], b[0]) + m(a[0], b[1]) + m(a[4], b2_19) + m(a[3], b3_19) + m(a[2], b4_19);
        let mut c2 =
            m(a[2], b[0]) + m(a[1], b[1]) + m(a[0], b[2]) + m(a[4], b3_19) + m(a[3], b4_19);
        let mut c3 = m(a[3], b[0]) + m(a[2], b[1]) + m(a[1], b[2]) + m(a[0], b[3]) + m(a[4], b4_19);
        let mut c4 = m(a[4], b[0]) + m(a[3], b[1]) + m(a[2], b[2]) + m(a[1], b[3]) + m(a[0], b[4]);
        debug_assert!(a[0] < (1 << 54));
        debug_assert!(b[0] < (1 << 54));
        debug_assert!(a[1] < (1 << 54));
        debug_assert!(b[1] < (1 << 54));
        debug_assert!(a[2] < (1 << 54));
        debug_assert!(b[2] < (1 << 54));
        debug_assert!(a[3] < (1 << 54));
        debug_assert!(b[3] < (1 << 54));
        debug_assert!(a[4] < (1 << 54));
        debug_assert!(b[4] < (1 << 54));
        let mut out = [0; 5];
        c1 += ((c0 >> 51) as u64) as u128;
        out[0] = (c0 as u64) & LOW_51_BITS;
        c2 += ((c1 >> 51) as u64) as u128;
        out[1] = (c1 as u64) & LOW_51_BITS;
        c3 += ((c2 >> 51) as u64) as u128;
        out[2] = (c2 as u64) & LOW_51_BITS;
        c4 += ((c3 >> 51) as u64) as u128;
        out[3] = (c3 as u64) & LOW_51_BITS;
        let carry = (c4 >> 51) as u64;
        out[4] = (c4 as u64) & LOW_51_BITS;
        out[0] += carry * 19;
        out[1] += out[0] >> 51;
        out[0] &= LOW_51_BITS;
        FieldElement51(out)
    }
}

impl Neg for &FieldElement51 {
    type Output = FieldElement51;

    fn neg(self) -> FieldElement51 {
        let mut output = *self;
        output.negate();
        output
    }
}

impl ConditionallySelectable for FieldElement51 {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Self([
            u64::conditional_select(&a.0[0], &b.0[0], choice),
            u64::conditional_select(&a.0[1], &b.0[1], choice),
            u64::conditional_select(&a.0[2], &b.0[2], choice),
            u64::conditional_select(&a.0[3], &b.0[3], choice),
            u64::conditional_select(&a.0[4], &b.0[4], choice),
        ])
    }

    fn conditional_swap(a: &mut Self, b: &mut Self, choice: Choice) {
        u64::conditional_swap(&mut a.0[0], &mut b.0[0], choice);
        u64::conditional_swap(&mut a.0[1], &mut b.0[1], choice);
        u64::conditional_swap(&mut a.0[2], &mut b.0[2], choice);
        u64::conditional_swap(&mut a.0[3], &mut b.0[3], choice);
        u64::conditional_swap(&mut a.0[4], &mut b.0[4], choice);
    }

    fn conditional_assign(&mut self, other: &Self, choice: Choice) {
        self.0[0].conditional_assign(&other.0[0], choice);
        self.0[1].conditional_assign(&other.0[1], choice);
        self.0[2].conditional_assign(&other.0[2], choice);
        self.0[3].conditional_assign(&other.0[3], choice);
        self.0[4].conditional_assign(&other.0[4], choice);
    }
}
