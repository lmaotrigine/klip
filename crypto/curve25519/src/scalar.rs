use crate::consts;
use core::{
    fmt::Debug,
    ops::{Add, Index, Mul},
};
use crypto_common::{
    constant_time::{Choice, ConstantTimeEq, OptionCt},
    erase::Erase,
};
use sha512::Sha512;

#[cfg(curve25519_bits = "32")]
type UnpackedScalar = crate::backends::serial::u32::scalar::Scalar29;
#[cfg(curve25519_bits = "64")]
type UnpackedScalar = crate::backends::serial::u64::scalar::Scalar52;

impl UnpackedScalar {
    const fn pack(&self) -> Scalar {
        Scalar {
            bytes: self.as_bytes(),
        }
    }
}

#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Clone, Copy, Hash)]
pub struct Scalar {
    pub(crate) bytes: [u8; 32],
}

impl Scalar {
    #[must_use]
    pub fn from_bytes_mod_order(bytes: [u8; 32]) -> Self {
        let s_unreduced = Self { bytes };
        let s = s_unreduced.reduce();
        debug_assert_eq!(s[31] >> 7, 0);
        s
    }

    pub(crate) fn from_bytes_mod_order_wide(input: &[u8; 64]) -> Self {
        UnpackedScalar::from_bytes_wide(input).pack()
    }

    #[must_use]
    pub fn from_canonical_bytes(bytes: [u8; 32]) -> OptionCt<Self> {
        let high_bit_unset = (bytes[31] >> 7).ct_eq(&0);
        let candidate = Self { bytes };
        OptionCt::new(candidate, high_bit_unset & candidate.is_canonical())
    }

    fn reduce(&self) -> Self {
        let x = self.unpack();
        let xr = UnpackedScalar::mul_internal(&x, &consts::R);
        let x_mod_l = UnpackedScalar::montgomery_reduce(&xr);
        x_mod_l.pack()
    }

    fn is_canonical(&self) -> Choice {
        self.ct_eq(&self.reduce())
    }

    #[must_use]
    pub fn from_hash(mut hash: Sha512) -> Self {
        let mut output = [0; 64];
        output.copy_from_slice(hash.finalize().as_slice());
        Self::from_bytes_mod_order_wide(&output)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    #[allow(clippy::cast_possible_wrap)]
    pub(crate) fn as_radix_16(&self) -> [i8; 64] {
        #[allow(clippy::identity_op)]
        #[inline(always)]
        const fn bot_half(x: u8) -> u8 {
            (x >> 0) & 15
        }
        #[inline(always)]
        const fn top_half(x: u8) -> u8 {
            (x >> 4) & 15
        }
        debug_assert!(self[31] <= 127);
        let mut output = [0; 64];

        for i in 0..32 {
            output[2 * i] = bot_half(self[i]) as i8;
            output[2 * i + 1] = top_half(self[i]) as i8;
        }
        for i in 0..63 {
            let carry = (output[i] + 8) >> 4;
            output[i] -= carry << 4;
            output[i + 1] += carry;
        }
        output
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub(crate) fn as_radix_2w(&self, w: usize) -> [i8; 64] {
        debug_assert!(w >= 4);
        debug_assert!(w <= 8);
        if w == 4 {
            return self.as_radix_16();
        }
        let mut scalar64x4 = [0; 4];
        read_le_u64_into(&self.bytes, &mut scalar64x4[0..4]);
        let radix = 1 << w;
        let window_mask = radix - 1;
        let mut carry = 0;
        let mut digits = [0; 64];
        let digits_count = (256 + w - 1) / w;
        #[allow(clippy::needless_range_loop)]
        for i in 0..digits_count {
            let bit_offset = i * w;
            let u64_idx = bit_offset / 64;
            let bit_idx = bit_offset % 64;
            let bit_buf = if bit_idx < 64 - w || u64_idx == 3 {
                scalar64x4[u64_idx] >> bit_idx
            } else {
                (scalar64x4[u64_idx] >> bit_idx) | (scalar64x4[u64_idx + 1] << (64 - bit_idx))
            };
            let coef = carry + (bit_buf & window_mask);
            carry = (coef + (radix / 2)) >> w;
            digits[i] = ((coef as i64) - (carry << w) as i64) as i8;
        }
        match w {
            8 => digits[digits_count] += carry as i8,
            _ => digits[digits_count - 1] += (carry << w) as i8,
        }
        digits
    }

    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn non_adjacent_form(&self, w: usize) -> [i8; 256] {
        debug_assert!(w >= 2);
        debug_assert!(w <= 8);
        let mut naf = [0; 256];
        let mut x_u64 = [0; 5];
        read_le_u64_into(&self.bytes, &mut x_u64[0..4]);
        let width = 1 << w;
        let window_mask = width - 1;
        let mut pos = 0;
        let mut carry = 0;
        while pos < 256 {
            let u64_idx = pos / 64;
            let bit_idx = pos % 64;
            let bit_buf: u64 = if bit_idx < 64 - w {
                x_u64[u64_idx] >> bit_idx
            } else {
                (x_u64[u64_idx] >> bit_idx) | (x_u64[u64_idx + 1] << (64 - bit_idx))
            };
            let window = carry + (bit_buf & window_mask);
            if window & 1 == 0 {
                pos += 1;
                continue;
            }
            if window < width / 2 {
                carry = 0;
                naf[pos] = window as i8;
            } else {
                carry = 1;
                naf[pos] = (window as i8).wrapping_sub(width as i8);
            }
            pos += w;
        }
        naf
    }

    pub(crate) fn unpack(&self) -> UnpackedScalar {
        UnpackedScalar::from_bytes(&self.bytes)
    }
}

impl Debug for Scalar {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            write!(f, "Scalar{{\n\tbytes: {:#?},\n}}", &self.bytes)
        } else {
            write!(f, "Scalar{{ bytes: {:?} }}", &self.bytes)
        }
    }
}

impl ConstantTimeEq for Scalar {
    fn ct_eq(&self, other: &Self) -> crypto_common::constant_time::Choice {
        self.bytes.ct_eq(&other.bytes)
    }
}

impl PartialEq for Scalar {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).into()
    }
}

impl Eq for Scalar {}

impl Index<usize> for Scalar {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &(self.bytes[index])
    }
}

impl Mul<Self> for Scalar {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        UnpackedScalar::mul(&self.unpack(), &rhs.unpack()).pack()
    }
}

impl Add<Self> for Scalar {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        UnpackedScalar::add(&self.unpack(), &rhs.unpack()).pack()
    }
}

impl Erase for Scalar {
    fn erase(&mut self) {
        self.bytes.erase();
    }
}

fn read_le_u64_into(src: &[u8], dst: &mut [u64]) {
    assert!(
        src.len() == 8 * dst.len(),
        "src.len() = {}, dst.len() = {}",
        src.len(),
        dst.len()
    );
    for (bytes, val) in src.chunks_exact(8).zip(dst.iter_mut()) {
        *val = u64::from_le_bytes(
            bytes
                .try_into()
                .expect("incorrect src length, should be 8 * dst.len()"),
        );
    }
}

#[must_use]
pub const fn clamp_integer(mut bytes: [u8; 32]) -> [u8; 32] {
    bytes[0] &= 0b1111_1000;
    bytes[31] &= 0b0111_1111;
    bytes[31] |= 0b0100_0000;
    bytes
}
