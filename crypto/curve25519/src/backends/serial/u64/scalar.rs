use super::consts;
use core::{
    fmt::Debug,
    ops::{Index, IndexMut},
};
use crypto_common::erase::Erase;

#[derive(Clone, Copy)]
pub struct Scalar52(pub [u64; 5]);

#[inline(always)]
fn m(x: u64, y: u64) -> u128 {
    u128::from(x) * u128::from(y)
}

impl Scalar52 {
    pub const ZERO: Self = Self([0, 0, 0, 0, 0]);

    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        let mut words = [0; 4];
        for i in 0..4 {
            for j in 0..8 {
                words[i] |= u64::from(bytes[i * 8 + j]) << (j * 8);
            }
        }
        let mask = (1 << 52) - 1;
        let top_mask = (1 << 48) - 1;
        let mut s = Self::ZERO;
        s[0] = words[0] & mask;
        s[1] = ((words[0] >> 52) | (words[1] << 12)) & mask;
        s[2] = ((words[1] >> 40) | (words[2] << 24)) & mask;
        s[3] = ((words[2] >> 28) | (words[3] << 36)) & mask;
        s[4] = (words[3] >> 16) & top_mask;
        s
    }

    pub fn from_bytes_wide(bytes: &[u8; 64]) -> Self {
        let mut words = [0; 8];
        for i in 0..8 {
            for j in 0..8 {
                words[i] |= u64::from(bytes[i * 8 + j]) << (j * 8);
            }
        }
        let mask = (1 << 52) - 1;
        let mut lo = Self::ZERO;
        let mut hi = Self::ZERO;
        lo[0] = words[0] & mask;
        lo[1] = ((words[0] >> 52) | (words[1] << 12)) & mask;
        lo[2] = ((words[1] >> 40) | (words[2] << 24)) & mask;
        lo[3] = ((words[2] >> 28) | (words[3] << 36)) & mask;
        lo[4] = ((words[3] >> 16) | (words[4] << 48)) & mask;
        hi[0] = (words[4] >> 4) & mask;
        hi[1] = ((words[4] >> 56) | (words[5] << 8)) & mask;
        hi[2] = ((words[5] >> 44) | (words[6] << 20)) & mask;
        hi[3] = ((words[6] >> 32) | (words[7] << 32)) & mask;
        hi[4] = words[7] >> 20;
        lo = Self::montgomery_mul(&lo, &consts::R);
        hi = Self::montgomery_mul(&hi, &consts::RR);
        Self::add(&lo, &hi)
    }

    #[allow(clippy::identity_op)]
    pub const fn as_bytes(&self) -> [u8; 32] {
        let mut s = [0; 32];
        s[0] = (self.0[0] >> 0) as u8;
        s[1] = (self.0[0] >> 8) as u8;
        s[2] = (self.0[0] >> 16) as u8;
        s[3] = (self.0[0] >> 24) as u8;
        s[4] = (self.0[0] >> 32) as u8;
        s[5] = (self.0[0] >> 40) as u8;
        s[6] = ((self.0[0] >> 48) | (self.0[1] << 4)) as u8;
        s[7] = (self.0[1] >> 4) as u8;
        s[8] = (self.0[1] >> 12) as u8;
        s[9] = (self.0[1] >> 20) as u8;
        s[10] = (self.0[1] >> 28) as u8;
        s[11] = (self.0[1] >> 36) as u8;
        s[12] = (self.0[1] >> 44) as u8;
        s[13] = (self.0[2] >> 0) as u8;
        s[14] = (self.0[2] >> 8) as u8;
        s[15] = (self.0[2] >> 16) as u8;
        s[16] = (self.0[2] >> 24) as u8;
        s[17] = (self.0[2] >> 32) as u8;
        s[18] = (self.0[2] >> 40) as u8;
        s[19] = ((self.0[2] >> 48) | (self.0[3] << 4)) as u8;
        s[20] = (self.0[3] >> 4) as u8;
        s[21] = (self.0[3] >> 12) as u8;
        s[22] = (self.0[3] >> 20) as u8;
        s[23] = (self.0[3] >> 28) as u8;
        s[24] = (self.0[3] >> 36) as u8;
        s[25] = (self.0[3] >> 44) as u8;
        s[26] = (self.0[4] >> 0) as u8;
        s[27] = (self.0[4] >> 8) as u8;
        s[28] = (self.0[4] >> 16) as u8;
        s[29] = (self.0[4] >> 24) as u8;
        s[30] = (self.0[4] >> 32) as u8;
        s[31] = (self.0[4] >> 40) as u8;
        s
    }

    pub fn add(a: &Self, b: &Self) -> Self {
        let mut sum = Self::ZERO;
        let mask = (1 << 52) - 1;
        let mut carry = 0;
        for i in 0..5 {
            carry = a[i] + b[i] + (carry >> 52);
            sum[i] = carry & mask;
        }
        Self::sub(&sum, &consts::L)
    }

    pub fn sub(a: &Self, b: &Self) -> Self {
        let mut difference = Self::ZERO;
        let mask = (1 << 52) - 1;
        let mut borrow = 0;
        for i in 0..5 {
            borrow = a[i].wrapping_sub(b[i] + (borrow >> 63));
            difference[i] = borrow & mask;
        }
        let underflow_mask = ((borrow >> 63) ^ 1).wrapping_sub(1);
        let mut carry = 0;
        for i in 0..5 {
            carry = (carry >> 52) + difference[i] + (consts::L[i] & underflow_mask);
            difference[i] = carry & mask;
        }
        difference
    }

    #[inline(always)]
    pub(crate) fn mul_internal(a: &Self, b: &Self) -> [u128; 9] {
        let mut z = [0; 9];
        z[0] = m(a[0], b[0]);
        z[1] = m(a[0], b[1]) + m(a[1], b[0]);
        z[2] = m(a[0], b[2]) + m(a[1], b[1]) + m(a[2], b[0]);
        z[3] = m(a[0], b[3]) + m(a[1], b[2]) + m(a[2], b[1]) + m(a[3], b[0]);
        z[4] = m(a[0], b[4]) + m(a[1], b[3]) + m(a[2], b[2]) + m(a[3], b[1]) + m(a[4], b[0]);
        z[5] = m(a[1], b[4]) + m(a[2], b[3]) + m(a[3], b[2]) + m(a[4], b[1]);
        z[6] = m(a[2], b[4]) + m(a[3], b[3]) + m(a[4], b[2]);
        z[7] = m(a[3], b[4]) + m(a[4], b[3]);
        z[8] = m(a[4], b[4]);
        z
    }

    // #[inline(always)]
    // fn square_internal(a: &Self) -> [u128; 9] {
    //     let aa = [a[0] * 2, a[1] * 2, a[2] * 2, a[3] * 2];
    //     [
    //         m(a[0], a[0]),
    //         m(aa[0], a[1]),
    //         m(aa[0], a[2]) + m(a[1], a[1]),
    //         m(aa[0], a[3]) + m(aa[1], a[2]),
    //         m(aa[0], a[4]) + m(aa[1], a[3]) + m(a[2], a[2]),
    //         m(aa[1], a[4]) + m(aa[2], a[3]),
    //         m(aa[2], a[4]) + m(a[3], a[3]),
    //         m(aa[3], a[4]),
    //         m(a[4], a[4]),
    //     ]
    // }

    #[inline(always)]
    pub(crate) fn montgomery_reduce(limbs: &[u128; 9]) -> Self {
        #[inline(always)]
        fn part1(sum: u128) -> (u128, u64) {
            let p = (sum as u64).wrapping_mul(consts::LFACTOR) & ((1 << 52) - 1);
            ((sum + m(p, consts::L[0])) >> 52, p)
        }
        #[inline(always)]
        const fn part2(sum: u128) -> (u128, u64) {
            let w = (sum as u64) & ((1 << 52) - 1);
            (sum >> 52, w)
        }
        let l = &consts::L;
        let (carry, n0) = part1(limbs[0]);
        let (carry, n1) = part1(carry + limbs[1] + m(n0, l[1]));
        let (carry, n2) = part1(carry + limbs[2] + m(n0, l[2]) + m(n1, l[1]));
        let (carry, n3) = part1(carry + limbs[3] + m(n1, l[2]) + m(n2, l[1]));
        let (carry, n4) = part1(carry + limbs[4] + m(n0, l[4]) + m(n2, l[2]) + m(n3, l[1]));
        let (carry, r0) = part2(carry + limbs[5] + m(n1, l[4]) + m(n3, l[2]) + m(n4, l[1]));
        let (carry, r1) = part2(carry + limbs[6] + m(n2, l[4]) + m(n4, l[2]));
        let (carry, r2) = part2(carry + limbs[7] + m(n3, l[4]));
        let (carry, r3) = part2(carry + limbs[8] + m(n4, l[4]));
        let r4 = carry as u64;
        Self::sub(&Self([r0, r1, r2, r3, r4]), l)
    }

    #[inline(never)]
    pub fn mul(a: &Self, b: &Self) -> Self {
        let ab = Self::montgomery_reduce(&Self::mul_internal(a, b));
        Self::montgomery_reduce(&Self::mul_internal(&ab, &consts::RR))
    }

    #[inline(never)]
    pub fn montgomery_mul(a: &Self, b: &Self) -> Self {
        Self::montgomery_reduce(&Self::mul_internal(a, b))
    }
}

impl Debug for Scalar52 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Scalar52: {:?}", &self.0[..])
    }
}

impl Erase for Scalar52 {
    fn erase(&mut self) {
        self.0.erase();
    }
}

impl Index<usize> for Scalar52 {
    type Output = u64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Scalar52 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}
