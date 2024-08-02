#![no_std]
#![allow(unexpected_cfgs)]
#![cfg_attr(
    all(curve25519_backend = "simd", nightly),
    feature(avx512_target_feature, stdarch_x86_avx512)
)]
#![deny(
    dead_code,
    deprecated,
    future_incompatible,
    missing_copy_implementations,
    missing_debug_implementations,
    nonstandard_style,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unused,
    clippy::all,
    clippy::pedantic,
    clippy::nursery
)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::inline_always
)]

use core::{
    fmt::Debug,
    ops::{Add, Mul, Neg},
};
use crypto_common::{
    constant_time::{Choice, ConditionallyNegatable, ConditionallySelectable, ConstantTimeEq},
    erase::Erase,
};

mod backends;
use backends::serial::curve_models::{AffineNielsPoint, ProjectiveNielsPoint, ProjectivePoint};
mod consts;
#[cfg(curve25519_report)]
mod report;
mod window;
use window::LookupTableRadix16;

pub mod scalar;
pub use scalar::Scalar;

#[cfg(curve25519_bits = "32")]
type FieldElement = backends::serial::u32::field::FieldElement2625;
#[cfg(curve25519_bits = "64")]
type FieldElement = backends::serial::u64::field::FieldElement51;

impl FieldElement {
    fn is_negative(&self) -> Choice {
        let bytes = self.as_bytes();
        (bytes[0] & 1).into()
    }

    fn pow25501(&self) -> (Self, Self) {
        let t0 = self.square();
        let t1 = t0.square().square();
        let t2 = self * &t1;
        let t3 = &t0 * &t2;
        let t4 = t3.square();
        let t5 = &t2 * &t4;
        let t6 = t5.pow2k(5);
        let t7 = &t6 * &t5;
        let t8 = t7.pow2k(10);
        let t9 = &t8 * &t7;
        let t10 = t9.pow2k(20);
        let t11 = &t10 * &t9;
        let t12 = t11.pow2k(10);
        let t13 = &t12 * &t7;
        let t14 = t13.pow2k(50);
        let t15 = &t14 * &t13;
        let t16 = t15.pow2k(100);
        let t17 = &t16 * &t15;
        let t18 = t17.pow2k(50);
        let t19 = &t18 * &t13;
        (t19, t3)
    }

    pub(crate) fn invert(&self) -> Self {
        let (t19, t3) = self.pow25501();
        let t20 = t19.pow2k(5);
        &t20 * &t3
    }

    fn pow_p58(&self) -> Self {
        let (t19, _) = self.pow25501();
        let t20 = t19.pow2k(2);
        // t21
        self * &t20
    }

    fn sqrt_ratio_i(u: &Self, v: &Self) -> (Choice, Self) {
        let v3 = &v.square() * v;
        let v7 = &v3.square() * v;
        let mut r = &(u * &v3) * &(u * &v7).pow_p58();
        let check = v * &r.square();
        let i = &consts::SQRT_M1;
        let correct_sign_sqrt = check.ct_eq(u);
        let flipped_sign_sqrt = check.ct_eq(&(-u));
        let flipped_sign_sqrt_i = check.ct_eq(&(&(-u) * i));
        let r_prime = &consts::SQRT_M1 * &r;
        r.conditional_assign(&r_prime, flipped_sign_sqrt | flipped_sign_sqrt_i);
        let r_is_negative = r.is_negative();
        r.conditional_negate(r_is_negative);
        let was_nonzero_square = correct_sign_sqrt | flipped_sign_sqrt;
        (was_nonzero_square, r)
    }
}

impl ConstantTimeEq for FieldElement {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.as_bytes().ct_eq(&other.as_bytes())
    }
}

impl PartialEq for FieldElement {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).into()
    }
}

impl Eq for FieldElement {}

#[derive(Clone, Copy)]
pub struct EdwardsPoint {
    pub(crate) x: FieldElement,
    pub(crate) y: FieldElement,
    pub(crate) z: FieldElement,
    pub(crate) t: FieldElement,
}

impl EdwardsPoint {
    fn as_projective_niels(&self) -> ProjectiveNielsPoint {
        ProjectiveNielsPoint {
            y_plus_x: &self.y + &self.x,
            y_minus_x: &self.y - &self.x,
            z: self.z,
            t2d: &self.t * &consts::EDWARDS_D2,
        }
    }

    const fn as_projective(&self) -> ProjectivePoint {
        ProjectivePoint {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }

    pub(crate) fn as_affine_niels(&self) -> AffineNielsPoint {
        let recip = self.z.invert();
        let x = &self.x * &recip;
        let y = &self.y * &recip;
        let xy2d = &(&x * &y) * &consts::EDWARDS_D2;
        AffineNielsPoint {
            y_plus_x: &y + &x,
            y_minus_x: &y - &x,
            xy2d,
        }
    }

    fn mul_by_cofactor(&self) -> Self {
        self.mul_by_pow_2(3)
    }

    fn mul_by_pow_2(&self, k: u32) -> Self {
        debug_assert!(k > 0);
        let mut r;
        let mut s = self.as_projective();
        for _ in 0..(k - 1) {
            r = s.double();
            s = r.as_projective();
        }
        s.double().as_extended()
    }

    #[must_use]
    pub fn is_small_order(&self) -> bool {
        self.mul_by_cofactor().ct_eq(&Self::default()).into()
    }

    #[must_use]
    pub fn mul_base(scalar: &Scalar) -> Self {
        scalar * consts::ED25519_BASEPOINT_TABLE
    }

    #[must_use]
    pub fn compress(&self) -> CompressedEdwardsY {
        let recip = self.z.invert();
        let x = &self.x * &recip;
        let y = &self.y * &recip;
        let mut s = y.as_bytes();
        s[31] ^= x.is_negative().to_u8() << 7;
        CompressedEdwardsY(s)
    }

    pub(crate) fn double(&self) -> Self {
        self.as_projective().double().as_extended()
    }

    #[must_use]
    pub fn vartime_double_scalar_mul_basepoint(a: &Scalar, big_a: &Self, b: &Scalar) -> Self {
        backends::vartime_double_base_mul(a, big_a, b)
    }
}

impl ConstantTimeEq for EdwardsPoint {
    fn ct_eq(&self, other: &Self) -> Choice {
        (&self.x * &other.z).ct_eq(&(&other.x * &self.z))
            & (&self.y * &other.z).ct_eq(&(&other.y * &self.z))
    }
}

impl PartialEq for EdwardsPoint {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).into()
    }
}

impl Eq for EdwardsPoint {}

impl Erase for EdwardsPoint {
    fn erase(&mut self) {
        self.x.erase();
        self.y = FieldElement::ONE;
        self.z = FieldElement::ONE;
        self.t.erase();
    }
}

impl Debug for EdwardsPoint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            write!(
                f,
                "EdwardsPoint{{\n\tX: {:#?},\n\tY: {:#?},\n\tZ: {:#?},\n\tT: {:#?}\n}}",
                &self.x, &self.y, &self.z, &self.t
            )
        } else {
            write!(
                f,
                "EdwardsPoint{{ X: {:?}, Y: {:?}, Z: {:?}, T: {:?} }}",
                &self.x, &self.y, &self.z, &self.t
            )
        }
    }
}

impl Default for EdwardsPoint {
    fn default() -> Self {
        Self {
            x: FieldElement::ZERO,
            y: FieldElement::ONE,
            z: FieldElement::ONE,
            t: FieldElement::ZERO,
        }
    }
}

impl<'a, 'b> Add<&'b EdwardsPoint> for &'a EdwardsPoint {
    type Output = EdwardsPoint;

    fn add(self, rhs: &'b EdwardsPoint) -> Self::Output {
        (self + &rhs.as_projective_niels()).as_extended()
    }
}

impl<'a> Neg for &'a EdwardsPoint {
    type Output = EdwardsPoint;

    fn neg(self) -> Self::Output {
        EdwardsPoint {
            x: -(&self.x),
            y: self.y,
            z: self.z,
            t: -(&self.t),
        }
    }
}

impl Neg for EdwardsPoint {
    type Output = Self;

    fn neg(self) -> Self::Output {
        -&self
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct EdwardsBasepointTable(pub(crate) [LookupTableRadix16<AffineNielsPoint>; 32]);

impl EdwardsBasepointTable {
    fn mul_base(&self, scalar: &crate::Scalar) -> EdwardsPoint {
        let a = scalar.as_radix_2w(4);
        let tables = &self.0;
        let mut p = EdwardsPoint::default();
        for i in (0..64).filter(|x| x % 2 == 1) {
            p = (&p + &tables[i / 2].select(a[i])).as_extended();
        }
        p = p.mul_by_pow_2(4);
        for i in (0..64).filter(|x| x % 2 == 0) {
            p = (&p + &tables[i / 2].select(a[i])).as_extended();
        }
        p
    }
}

impl<'a, 'b> Mul<&'a EdwardsBasepointTable> for &'b crate::Scalar {
    type Output = EdwardsPoint;

    fn mul(self, table: &'a EdwardsBasepointTable) -> Self::Output {
        table.mul_base(self)
    }
}

impl Debug for EdwardsBasepointTable {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "EdwardsBasepointTable({:?})", &self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompressedEdwardsY(pub [u8; 32]);

impl CompressedEdwardsY {
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    #[must_use]
    pub const fn to_bytes(&self) -> [u8; 32] {
        self.0
    }

    #[must_use]
    pub fn decompress(&self) -> Option<EdwardsPoint> {
        let (is_valid_y_coord, x, y, z) = decompress::step_1(self);
        if is_valid_y_coord.into() {
            Some(decompress::step_2(self, x, y, z))
        } else {
            None
        }
    }
}

impl ConstantTimeEq for CompressedEdwardsY {
    fn ct_eq(&self, other: &Self) -> crypto_common::constant_time::Choice {
        self.as_bytes().ct_eq(other.as_bytes())
    }
}

impl Erase for CompressedEdwardsY {
    fn erase(&mut self) {
        self.0.erase();
        self.0[0] = 1;
    }
}

impl Default for CompressedEdwardsY {
    fn default() -> Self {
        Self([
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ])
    }
}

impl Debug for CompressedEdwardsY {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CompressedEdwardsY: {:?}", self.as_bytes())
    }
}

mod decompress {
    use crate::{consts, CompressedEdwardsY, EdwardsPoint, FieldElement};
    use crypto_common::constant_time::{Choice, ConditionallyNegatable};

    #[allow(clippy::many_single_char_names)]
    pub fn step_1(repr: &CompressedEdwardsY) -> (Choice, FieldElement, FieldElement, FieldElement) {
        let y = FieldElement::from_bytes(repr.as_bytes());
        let z = FieldElement::ONE;
        let yy = y.square();
        let u = &yy - &z;
        let v = &(&yy * &consts::EDWARDS_D) + &z;
        let (is_valid_y_coord, x) = FieldElement::sqrt_ratio_i(&u, &v);
        (is_valid_y_coord, x, y, z)
    }

    pub fn step_2(
        repr: &CompressedEdwardsY,
        mut x: FieldElement,
        y: FieldElement,
        z: FieldElement,
    ) -> EdwardsPoint {
        let compressed_sign_bit = Choice::from(repr.as_bytes()[31] >> 7);
        x.conditional_negate(compressed_sign_bit);
        EdwardsPoint {
            x,
            y,
            z,
            t: &x * &y,
        }
    }
}
