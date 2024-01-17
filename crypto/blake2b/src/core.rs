use crate::{
    simd::u64x4,
    traits::{AsBytes, SliceExt},
    IV, SIGMA,
};
use crypto_common::constant_time::ConstantTimeEq;

#[derive(Debug, Clone, Copy)]
pub struct Result {
    h: [u64x4; 2],
    nn: usize,
}

impl Result {
    #[inline]
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.h.as_bytes()[..self.nn]
    }

    #[inline]
    #[must_use]
    #[allow(clippy::len_without_is_empty)]
    pub const fn len(&self) -> usize {
        self.nn
    }
}

impl AsRef<[u8]> for Result {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl PartialEq for Result {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes().ct_eq(other.as_bytes()).into()
    }
}

impl PartialEq<[u8]> for Result {
    #[inline]
    fn eq(&self, other: &[u8]) -> bool {
        self.as_bytes().ct_eq(other).into()
    }
}

impl Eq for Result {}

#[allow(missing_copy_implementations)]
#[derive(Clone)]
pub struct Blake2b {
    m: [u64; 16],
    h: [u64x4; 2],
    t: u64,
    nn: usize,
}

impl core::fmt::Debug for Blake2b {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "State {{ .. }}")
    }
}

impl Blake2b {
    #[must_use]
    pub fn new(output_size: usize) -> Self {
        Self::new_with_params(&[], &[], &[], output_size)
    }

    #[must_use]
    pub fn new_with_params(salt: &[u8], personal: &[u8], key: &[u8], output_size: usize) -> Self {
        let key_size = key.len();
        assert!(key_size <= 64);
        assert!(output_size <= 64);
        assert!(salt.len() <= 16);
        assert!(personal.len() <= 16);
        let mut p = [0; 8];
        p[0] = 0x0101_0000 ^ ((key_size as u64) << 8) ^ (output_size as u64);
        if salt.len() < 16 {
            let mut padded_salt = [0; 16];
            padded_salt[..salt.len()].copy_from_slice(salt);
            p[4] = u64::from_le_bytes(padded_salt[0..8].try_into().unwrap());
            p[5] = u64::from_le_bytes(padded_salt[8..padded_salt.len()].try_into().unwrap());
        } else {
            p[4] = u64::from_le_bytes(salt[0..salt.len() / 2].try_into().unwrap());
            p[5] = u64::from_le_bytes(salt[salt.len() / 2..salt.len()].try_into().unwrap());
        }
        if personal.len() < 16 {
            let mut padded_personal = [0; 16];
            padded_personal[..personal.len()].copy_from_slice(personal);
            p[6] = u64::from_le_bytes(padded_personal[0..8].try_into().unwrap());
            p[7] = u64::from_le_bytes(
                padded_personal[8..padded_personal.len()]
                    .try_into()
                    .unwrap(),
            );
        } else {
            p[6] = u64::from_le_bytes(personal[0..8].try_into().unwrap());
            p[7] = u64::from_le_bytes(personal[8..personal.len()].try_into().unwrap());
        }
        let h = [
            iv0() ^ u64x4::new(p[0], p[1], p[2], p[3]),
            iv1() ^ u64x4::new(p[4], p[5], p[6], p[7]),
        ];
        let mut m = [0; 16];
        let t = if key_size > 0 {
            m.as_mut_bytes().copy_bytes_from(key);
            128
        } else {
            0
        };
        Self {
            h,
            m,
            t,
            nn: output_size,
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        let mut rest = data;
        let off = (self.t % 128) as usize;
        if off != 0 || self.t == 0 {
            let len = rest.len().min(128 - off);
            let part = &rest[..len];
            rest = &rest[part.len()..];
            self.m.as_mut_bytes()[off..].copy_bytes_from(part);
            self.t = self
                .t
                .checked_add(part.len() as u64)
                .expect("hash data length overflow");
        }
        while rest.len() >= 128 {
            self.compress(0, 0);
            let part = &rest[..128];
            rest = &rest[part.len()..];
            self.m.as_mut_bytes().copy_bytes_from(part);
            self.t = self
                .t
                .checked_add(part.len() as u64)
                .expect("hash data length overflow");
        }
        if !rest.is_empty() {
            self.compress(0, 0);
            self.m.as_mut_bytes().copy_bytes_from(rest);
            self.t = self
                .t
                .checked_add(rest.len() as u64)
                .expect("hash data length overflow");
        }
    }

    fn finalize_with_flag(&mut self, f1: u64) {
        let off = (self.t % 128) as usize;
        if off != 0 {
            self.m.as_mut_bytes()[off..].set_bytes(0);
        }
        self.compress(!0, f1);
    }

    #[inline]
    #[must_use]
    pub fn finalize(mut self) -> Result {
        self.finalize_with_flag(0);
        self.into_result()
    }

    #[inline]
    const fn into_result(self) -> Result {
        Result {
            h: [self.h[0].to_le(), self.h[1].to_le()],
            nn: self.nn,
        }
    }

    #[inline(always)]
    fn quarter_round(v: &mut [u64x4; 4], rd: u32, rb: u32, m: u64x4) {
        v[0] = v[0].wrapping_add(v[1]).wrapping_add(m.from_le());
        v[3] = (v[3] ^ v[0]).rotate_right(rd);
        v[2] = v[2].wrapping_add(v[3]);
        v[1] = (v[1] ^ v[2]).rotate_right(rb);
    }

    #[inline(always)]
    fn shuffle(v: &mut [u64x4; 4]) {
        v[1] = v[1].shuffle_left_1();
        v[2] = v[2].shuffle_left_2();
        v[3] = v[3].shuffle_left_3();
    }

    #[inline(always)]
    fn unshuffle(v: &mut [u64x4; 4]) {
        v[1] = v[1].shuffle_right_1();
        v[2] = v[2].shuffle_right_2();
        v[3] = v[3].shuffle_right_3();
    }

    #[inline(always)]
    fn round(v: &mut [u64x4; 4], m: &[u64; 16], s: &[usize; 16]) {
        Self::quarter_round(v, 32, 24, u64x4::gather(m, s[0], s[2], s[4], s[6]));
        Self::quarter_round(v, 16, 63, u64x4::gather(m, s[1], s[3], s[5], s[7]));
        Self::shuffle(v);
        Self::quarter_round(v, 32, 24, u64x4::gather(m, s[8], s[10], s[12], s[14]));
        Self::quarter_round(v, 16, 63, u64x4::gather(m, s[9], s[11], s[13], s[15]));
        Self::unshuffle(v);
    }

    fn compress(&mut self, f0: u64, f1: u64) {
        let m = &self.m;
        let h = &mut self.h;
        let t0 = self.t;
        let t1 = 0;
        let mut v = [h[0], h[1], iv0(), iv1() ^ u64x4::new(t0, t1, f0, f1)];
        Self::round(&mut v, m, &SIGMA[0]);
        Self::round(&mut v, m, &SIGMA[1]);
        Self::round(&mut v, m, &SIGMA[2]);
        Self::round(&mut v, m, &SIGMA[3]);
        Self::round(&mut v, m, &SIGMA[4]);
        Self::round(&mut v, m, &SIGMA[5]);
        Self::round(&mut v, m, &SIGMA[6]);
        Self::round(&mut v, m, &SIGMA[7]);
        Self::round(&mut v, m, &SIGMA[8]);
        Self::round(&mut v, m, &SIGMA[9]);
        Self::round(&mut v, m, &SIGMA[0]);
        Self::round(&mut v, m, &SIGMA[1]);
        h[0] = h[0] ^ (v[0] ^ v[2]);
        h[1] = h[1] ^ (v[1] ^ v[3]);
    }
}

impl Default for Blake2b {
    fn default() -> Self {
        Self::new(64)
    }
}

#[inline(always)]
const fn iv0() -> u64x4 {
    u64x4::new(IV[0], IV[1], IV[2], IV[3])
}

#[inline(always)]
const fn iv1() -> u64x4 {
    u64x4::new(IV[4], IV[5], IV[6], IV[7])
}

#[must_use]
pub fn blake2b(
    salt: &[u8],
    personal: &[u8],
    key: &[u8],
    output_size: usize,
    data: &[u8],
) -> Result {
    let mut state = Blake2b::new_with_params(salt, personal, key, output_size);
    state.update(data);
    state.finalize()
}
