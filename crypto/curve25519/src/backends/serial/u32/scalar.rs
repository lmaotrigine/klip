use core::{
    fmt::Debug,
    ops::{Index, IndexMut},
};
use crypto_common::erase::Erase;

use super::consts;

#[derive(Clone, Copy)]
pub struct Scalar29(pub [u32; 9]);

impl Debug for Scalar29 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Scalar29: {:?}", &self.0[..])
    }
}

impl Erase for Scalar29 {
    fn erase(&mut self) {
        self.0.erase();
    }
}

impl Index<usize> for Scalar29 {
    type Output = u32;

    fn index(&self, index: usize) -> &Self::Output {
        &(self.0[index])
    }
}

impl IndexMut<usize> for Scalar29 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut (self.0[index])
    }
}

#[inline(always)]
const fn m(x: u32, y: u32) -> u64 {
    (x as u64) * (y as u64)
}

impl Scalar29 {
    pub const ZERO: Self = Self([0, 0, 0, 0, 0, 0, 0, 0, 0]);

    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        let mut words = [0; 8];
        for i in 0..8 {
            for j in 0..4 {
                words[i] |= (bytes[(i * 4) + j] as u32) << (j * 8);
            }
        }
        let mask = (1 << 29) - 1;
        let top_mask = (1 << 24) - 1;
        let mut s = Self::ZERO;
        s[0] = words[0] & mask;
        s[1] = ((words[0] >> 29) | (words[1] << 3)) & mask;
        s[2] = ((words[1] >> 26) | (words[2] << 6)) & mask;
        s[3] = ((words[2] >> 23) | (words[3] << 9)) & mask;
        s[4] = ((words[3] >> 20) | (words[4] << 12)) & mask;
        s[5] = ((words[4] >> 17) | (words[5] << 15)) & mask;
        s[6] = ((words[5] >> 14) | (words[6] << 18)) & mask;
        s[7] = ((words[6] >> 11) | (words[7] << 21)) & mask;
        s[8] = (words[7] >> 8) & top_mask;
        s
    }

    pub fn from_bytes_wide(bytes: &[u8; 64]) -> Self {
        let mut words = [0; 16];
        for i in 0..16 {
            for j in 0..4 {
                words[i] |= (bytes[i * 4 + j] as u32) << (j * 8);
            }
        }
        let mask = (1 << 29) - 1;
        let mut lo = Self::ZERO;
        let mut hi = Self::ZERO;
        lo[0] = words[0] & mask;
        lo[1] = ((words[0] >> 29) | (words[1] << 3)) & mask;
        lo[2] = ((words[1] >> 26) | (words[2] << 6)) & mask;
        lo[3] = ((words[2] >> 23) | (words[3] << 9)) & mask;
        lo[4] = ((words[3] >> 20) | (words[4] << 12)) & mask;
        lo[5] = ((words[4] >> 17) | (words[5] << 15)) & mask;
        lo[6] = ((words[5] >> 14) | (words[6] << 18)) & mask;
        lo[7] = ((words[6] >> 11) | (words[7] << 21)) & mask;
        lo[8] = ((words[7] >> 8) | (words[8] << 24)) & mask;
        hi[0] = ((words[8] >> 5) | (words[9] << 27)) & mask;
        hi[1] = (words[9] >> 2) & mask;
        hi[2] = ((words[9] >> 31) | (words[10] << 1)) & mask;
        hi[3] = ((words[10] >> 28) | (words[11] << 4)) & mask;
        hi[4] = ((words[11] >> 25) | (words[12] << 7)) & mask;
        hi[5] = ((words[12] >> 22) | (words[13] << 10)) & mask;
        hi[6] = ((words[13] >> 19) | (words[14] << 13)) & mask;
        hi[7] = ((words[14] >> 16) | (words[15] << 16)) & mask;
        hi[8] = words[15] >> 13;
        lo = Self::montgomery_mul(&lo, &consts::R);
        hi = Self::montgomery_mul(&hi, &consts::RR);
        Self::add(&hi, &lo)
    }

    #[allow(clippy::identity_op)]
    pub const fn as_bytes(&self) -> [u8; 32] {
        let mut s = [0; 32];
        s[0] = (self.0[0] >> 0) as u8;
        s[1] = (self.0[0] >> 8) as u8;
        s[2] = (self.0[0] >> 16) as u8;
        s[3] = ((self.0[0] >> 24) | (self.0[1] << 5)) as u8;
        s[4] = (self.0[1] >> 3) as u8;
        s[5] = (self.0[1] >> 11) as u8;
        s[6] = (self.0[1] >> 19) as u8;
        s[7] = ((self.0[1] >> 27) | (self.0[2] << 2)) as u8;
        s[8] = (self.0[2] >> 6) as u8;
        s[9] = (self.0[2] >> 14) as u8;
        s[10] = ((self.0[2] >> 22) | (self.0[3] << 7)) as u8;
        s[11] = (self.0[3] >> 1) as u8;
        s[12] = (self.0[3] >> 9) as u8;
        s[13] = (self.0[3] >> 17) as u8;
        s[14] = ((self.0[3] >> 25) | (self.0[4] << 4)) as u8;
        s[15] = (self.0[4] >> 4) as u8;
        s[16] = (self.0[4] >> 12) as u8;
        s[17] = (self.0[4] >> 20) as u8;
        s[18] = ((self.0[4] >> 28) | (self.0[5] << 1)) as u8;
        s[19] = (self.0[5] >> 7) as u8;
        s[20] = (self.0[5] >> 15) as u8;
        s[21] = ((self.0[5] >> 23) | (self.0[6] << 6)) as u8;
        s[22] = (self.0[6] >> 2) as u8;
        s[23] = (self.0[6] >> 10) as u8;
        s[24] = (self.0[6] >> 18) as u8;
        s[25] = ((self.0[6] >> 26) | (self.0[7] << 3)) as u8;
        s[26] = (self.0[7] >> 5) as u8;
        s[27] = (self.0[7] >> 13) as u8;
        s[28] = (self.0[7] >> 21) as u8;
        s[29] = (self.0[8] >> 0) as u8;
        s[30] = (self.0[8] >> 8) as u8;
        s[31] = (self.0[8] >> 16) as u8;
        s
    }

    pub fn add(a: &Self, b: &Self) -> Self {
        let mut sum = Self::ZERO;
        let mask = (1 << 29) - 1;
        let mut carry = 0;
        for i in 0..9 {
            carry = a[i] + b[i] + (carry >> 29);
            sum[i] = carry & mask;
        }
        Self::sub(&sum, &consts::L)
    }

    pub fn sub(a: &Self, b: &Self) -> Self {
        let mut difference = Self::ZERO;
        let mask = (1 << 29) - 1;
        let mut borrow = 0;
        for i in 0..9 {
            borrow = a[i].wrapping_sub(b[i] + (borrow >> 31));
            difference[i] = borrow & mask;
        }
        let underflow_mask = ((borrow >> 31) ^ 1).wrapping_sub(1);
        let mut carry = 0;
        for i in 0..9 {
            carry = (carry >> 29) + difference[i] + (consts::L[i] & underflow_mask);
            difference[i] = carry & mask;
        }
        difference
    }

    #[inline(always)]
    pub(crate) fn mul_internal(a: &Self, b: &Self) -> [u64; 17] {
        let mut z = [0; 17];
        z[0] = m(a[0], b[0]);
        z[1] = m(a[0], b[1]) + m(a[1], b[0]);
        z[2] = m(a[0], b[2]) + m(a[1], b[1]) + m(a[2], b[0]);
        z[3] = m(a[0], b[3]) + m(a[1], b[2]) + m(a[2], b[1]) + m(a[3], b[0]);
        z[4] = m(a[0], b[4]) + m(a[1], b[3]) + m(a[2], b[2]) + m(a[3], b[1]) + m(a[4], b[0]);
        z[5] = m(a[1], b[4]) + m(a[2], b[3]) + m(a[3], b[2]) + m(a[4], b[1]);
        z[6] = m(a[2], b[4]) + m(a[3], b[3]) + m(a[4], b[2]);
        z[7] = m(a[3], b[4]) + m(a[4], b[3]);
        z[8] = (m(a[4], b[4])).wrapping_sub(z[3]);
        z[10] = z[5].wrapping_sub(m(a[5], b[5]));
        z[11] = z[6].wrapping_sub(m(a[5], b[6]) + m(a[6], b[5]));
        z[12] = z[7].wrapping_sub(m(a[5], b[7]) + m(a[6], b[6]) + m(a[7], b[5]));
        z[13] = m(a[5], b[8]) + m(a[6], b[7]) + m(a[7], b[6]) + m(a[8], b[5]);
        z[14] = m(a[6], b[8]) + m(a[7], b[7]) + m(a[8], b[6]);
        z[15] = m(a[7], b[8]) + m(a[8], b[7]);
        z[16] = m(a[8], b[8]);
        z[5] = z[10].wrapping_sub(z[0]);
        z[6] = z[11].wrapping_sub(z[1]);
        z[7] = z[12].wrapping_sub(z[2]);
        z[8] = z[8].wrapping_sub(z[13]);
        z[9] = z[14].wrapping_add(z[4]);
        z[10] = z[15].wrapping_add(z[10]);
        z[11] = z[16].wrapping_add(z[11]);
        let aa = [a[0] + a[5], a[1] + a[6], a[2] + a[7], a[3] + a[8]];
        let bb = [b[0] + b[5], b[1] + b[6], b[2] + b[7], b[3] + b[8]];
        z[5] = (m(aa[0], bb[0])).wrapping_add(z[5]);
        z[6] = (m(aa[0], bb[1]) + m(aa[1], bb[0])).wrapping_add(z[6]);
        z[7] = (m(aa[0], bb[2]) + m(aa[1], bb[1]) + m(aa[2], bb[0])).wrapping_add(z[7]);
        z[8] = (m(aa[0], bb[3]) + m(aa[1], bb[2]) + m(aa[2], bb[1]) + m(aa[3], bb[0]))
            .wrapping_add(z[8]);
        z[9] =
            (m(aa[0], b[4]) + m(aa[1], bb[3]) + m(aa[2], bb[2]) + m(aa[3], bb[1]) + m(a[4], bb[0]))
                .wrapping_sub(z[9]);
        z[10] = (m(aa[1], b[4]) + m(aa[2], bb[3]) + m(aa[3], bb[2]) + m(a[4], bb[1]))
            .wrapping_sub(z[10]);
        z[11] = (m(aa[2], b[4]) + m(aa[3], bb[3]) + m(a[4], bb[2])).wrapping_sub(z[11]);
        z[12] = (m(aa[3], b[4]) + m(a[4], bb[3])).wrapping_sub(z[12]);
        z
    }

    // #[inline(always)]
    // fn square_internal(a: &Self) -> [u64; 17] {
    //     let aa = [
    //         a[0] * 2,
    //         a[1] * 2,
    //         a[2] * 2,
    //         a[3] * 2,
    //         a[4] * 2,
    //         a[5] * 2,
    //         a[6] * 2,
    //         a[7] * 2,
    //     ];
    //     [
    //         m(a[0], a[0]),
    //         m(aa[0], a[1]),
    //         m(aa[0], a[2]) + m(a[1], a[1]),
    //         m(aa[0], a[3]) + m(aa[1], a[2]),
    //         m(aa[0], a[4]) + m(aa[1], a[3]) + m(a[2], a[2]),
    //         m(aa[0], a[5]) + m(aa[1], a[4]) + m(aa[2], a[3]),
    //         m(aa[0], a[6]) + m(aa[1], a[5]) + m(aa[2], a[4]) + m(a[3], a[3]),
    //         m(aa[0], a[7]) + m(aa[1], a[6]) + m(aa[2], a[5]) + m(aa[3], a[4]),
    //         m(aa[0], a[8]) + m(aa[1], a[7]) + m(aa[2], a[6]) + m(aa[3], a[5]) +
    // m(a[4], a[4]),         m(aa[1], a[8]) + m(aa[2], a[7]) + m(aa[3], a[6]) +
    // m(aa[4], a[5]),         m(aa[2], a[8]) + m(aa[3], a[7]) + m(aa[4], a[6])
    // + m(a[5], a[5]),         m(aa[3], a[8]) + m(aa[4], a[7]) + m(aa[5],
    // a[6]),         m(aa[4], a[8]) + m(aa[5], a[7]) + m(a[6], a[6]),
    //         m(aa[5], a[8]) + m(aa[6], a[7]),
    //         m(aa[6], a[8]) + m(a[7], a[7]),
    //         m(aa[7], a[8]),
    //         m(a[8], a[8]),
    //     ]
    // }

    #[inline(always)]
    pub(crate) fn montgomery_reduce(limbs: &[u64; 17]) -> Self {
        #[inline(always)]
        fn part1(sum: u64) -> (u64, u32) {
            let p = (sum as u32).wrapping_mul(consts::LFACTOR) & ((1 << 29) - 1);
            ((sum + m(p, consts::L[0])) >> 29, p)
        }
        #[inline(always)]
        const fn part2(sum: u64) -> (u64, u32) {
            let w = (sum as u32) & ((1 << 29) - 1);
            (sum >> 29, w)
        }
        let l = &consts::L;
        let (carry, n0) = part1(limbs[0]);
        let (carry, n1) = part1(carry + limbs[1] + m(n0, l[1]));
        let (carry, n2) = part1(carry + limbs[2] + m(n0, l[2]) + m(n1, l[1]));
        let (carry, n3) = part1(carry + limbs[3] + m(n0, l[3]) + m(n1, l[2]) + m(n2, l[1]));
        let (carry, n4) =
            part1(carry + limbs[4] + m(n0, l[4]) + m(n1, l[3]) + m(n2, l[2]) + m(n3, l[1]));
        let (carry, n5) =
            part1(carry + limbs[5] + m(n1, l[4]) + m(n2, l[3]) + m(n3, l[2]) + m(n4, l[1]));
        let (carry, n6) =
            part1(carry + limbs[6] + m(n2, l[4]) + m(n3, l[3]) + m(n4, l[2]) + m(n5, l[1]));
        let (carry, n7) =
            part1(carry + limbs[7] + m(n3, l[4]) + m(n4, l[3]) + m(n5, l[2]) + m(n6, l[1]));
        let (carry, n8) = part1(
            carry + limbs[8] + m(n0, l[8]) + m(n4, l[4]) + m(n5, l[3]) + m(n6, l[2]) + m(n7, l[1]),
        );
        let (carry, r0) = part2(
            carry + limbs[9] + m(n1, l[8]) + m(n5, l[4]) + m(n6, l[3]) + m(n7, l[2]) + m(n8, l[1]),
        );
        let (carry, r1) =
            part2(carry + limbs[10] + m(n2, l[8]) + m(n6, l[4]) + m(n7, l[3]) + m(n8, l[2]));
        let (carry, r2) = part2(carry + limbs[11] + m(n3, l[8]) + m(n7, l[4]) + m(n8, l[3]));
        let (carry, r3) = part2(carry + limbs[12] + m(n4, l[8]) + m(n8, l[4]));
        let (carry, r4) = part2(carry + limbs[13] + m(n5, l[8]));
        let (carry, r5) = part2(carry + limbs[14] + m(n6, l[8]));
        let (carry, r6) = part2(carry + limbs[15] + m(n7, l[8]));
        let (carry, r7) = part2(carry + limbs[16] + m(n8, l[8]));
        let r8 = carry as u32;
        Self::sub(&Self([r0, r1, r2, r3, r4, r5, r6, r7, r8]), l)
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
