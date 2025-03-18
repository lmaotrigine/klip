use crate::{EdwardsPoint, FieldElement};
use core::{
    fmt::Debug,
    ops::{Add, Neg, Sub},
};
use crypto_common::{constant_time::ConditionallySelectable, erase::Erase};

#[derive(Clone, Copy)]
pub struct ProjectivePoint {
    pub x: FieldElement,
    pub y: FieldElement,
    pub z: FieldElement,
}

impl ProjectivePoint {
    pub fn double(&self) -> CompletedPoint {
        let xx = self.x.square();
        let yy = self.y.square();
        let zz2 = self.z.square2();
        let x_plus_y = &self.x + &self.y;
        let x_plus_y_squared = x_plus_y.square();
        let yy_plus_xx = &yy + &xx;
        let yy_minus_xx = &yy - &xx;
        CompletedPoint {
            x: &x_plus_y_squared - &yy_plus_xx,
            y: yy_plus_xx,
            z: yy_minus_xx,
            t: &zz2 - &yy_minus_xx,
        }
    }

    pub fn as_extended(&self) -> EdwardsPoint {
        EdwardsPoint {
            x: &self.x * &self.z,
            y: &self.y * &self.z,
            z: self.z.square(),
            t: &self.x * &self.y,
        }
    }
}

impl Default for ProjectivePoint {
    fn default() -> Self {
        Self {
            x: FieldElement::ZERO,
            y: FieldElement::ONE,
            z: FieldElement::ONE,
        }
    }
}

impl Debug for ProjectivePoint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            write!(
                f,
                "ProjectivePoint{{\n\tX: {:#?},\n\tY: {:#?},\n\tZ: {:#?}\n}}",
                &self.x, &self.y, &self.z
            )
        } else {
            write!(
                f,
                "ProjectivePoint{{ X: {:?}, Y: {:?}, Z: {:?} }}",
                &self.x, &self.y, &self.z
            )
        }
    }
}

#[derive(Clone, Copy)]
pub struct CompletedPoint {
    pub x: FieldElement,
    pub y: FieldElement,
    pub z: FieldElement,
    pub t: FieldElement,
}

impl CompletedPoint {
    pub fn as_extended(&self) -> EdwardsPoint {
        EdwardsPoint {
            x: &self.x * &self.t,
            y: &self.y * &self.z,
            z: &self.z * &self.t,
            t: &self.x * &self.y,
        }
    }

    pub fn as_projective(&self) -> ProjectivePoint {
        ProjectivePoint {
            x: &self.x * &self.t,
            y: &self.y * &self.z,
            z: &self.z * &self.t,
        }
    }
}

impl Debug for CompletedPoint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            write!(
                f,
                "CompletedPoint{{\n\tX: {:#?},\n\tY: {:#?},\n\tZ: {:#?},\n\tT: {:#?}\n}}",
                &self.x, &self.y, &self.z, &self.t
            )
        } else {
            write!(
                f,
                "CompletedPoint{{ X: {:?}, Y: {:?}, Z: {:?}, T: {:?} }}",
                &self.x, &self.y, &self.z, &self.t
            )
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AffineNielsPoint {
    pub y_plus_x: FieldElement,
    pub y_minus_x: FieldElement,
    pub xy2d: FieldElement,
}

impl Erase for AffineNielsPoint {
    fn erase(&mut self) {
        self.y_plus_x.erase();
        self.y_minus_x.erase();
        self.xy2d.erase();
    }
}

impl Default for AffineNielsPoint {
    fn default() -> Self {
        Self {
            y_plus_x: FieldElement::ONE,
            y_minus_x: FieldElement::ONE,
            xy2d: FieldElement::ZERO,
        }
    }
}

impl ConditionallySelectable for AffineNielsPoint {
    fn conditional_select(
        a: &Self,
        b: &Self,
        choice: crypto_common::constant_time::Choice,
    ) -> Self {
        Self {
            y_plus_x: FieldElement::conditional_select(&a.y_plus_x, &b.y_plus_x, choice),
            y_minus_x: FieldElement::conditional_select(&a.y_minus_x, &b.y_minus_x, choice),
            xy2d: FieldElement::conditional_select(&a.xy2d, &b.xy2d, choice),
        }
    }

    fn conditional_assign(&mut self, other: &Self, choice: crypto_common::constant_time::Choice) {
        self.y_plus_x.conditional_assign(&other.y_plus_x, choice);
        self.y_minus_x.conditional_assign(&other.y_minus_x, choice);
        self.xy2d.conditional_assign(&other.xy2d, choice);
    }
}

impl Neg for &AffineNielsPoint {
    type Output = AffineNielsPoint;

    fn neg(self) -> Self::Output {
        AffineNielsPoint {
            y_plus_x: self.y_minus_x,
            y_minus_x: self.y_plus_x,
            xy2d: -(&self.xy2d),
        }
    }
}

impl Debug for AffineNielsPoint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            write!(
                f,
                "AffineNielsPoint{{\n\ty_plus_x: {:#?},\n\ty_minus_x: {:#?},\n\txy2d: {:#?}\n}}",
                &self.y_plus_x, &self.y_minus_x, &self.xy2d
            )
        } else {
            write!(
                f,
                "AffineNielsPoint{{ y_plus_x: {:?}, y_minus_x: {:?}, xy2d: {:?} }}",
                &self.y_plus_x, &self.y_minus_x, &self.xy2d
            )
        }
    }
}

#[derive(Clone, Copy)]
pub struct ProjectiveNielsPoint {
    pub y_plus_x: FieldElement,
    pub y_minus_x: FieldElement,
    pub z: FieldElement,
    pub t2d: FieldElement,
}

impl Debug for ProjectiveNielsPoint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            write!(
                f,
                "ProjectiveNielsPoint{{\n\tY_plus_X: {:#?},\n\tY_minus_X: {:#?},\n\tZ: \
                 {:#?},\n\tT2d: {:#?}\n}}",
                &self.y_plus_x, &self.y_minus_x, &self.z, &self.t2d
            )
        } else {
            write!(
                f,
                "ProjectiveNielsPoint{{ Y_plus_X: {:?}, Y_minus_X: {:?}, Z: {:?}, T2d: {:?} }}",
                &self.y_plus_x, &self.y_minus_x, &self.z, &self.t2d
            )
        }
    }
}

impl Erase for ProjectiveNielsPoint {
    fn erase(&mut self) {
        self.y_plus_x.erase();
        self.y_minus_x.erase();
        self.z.erase();
        self.t2d.erase();
    }
}

impl<'b> Add<&'b ProjectiveNielsPoint> for &EdwardsPoint {
    type Output = CompletedPoint;

    fn add(self, rhs: &'b ProjectiveNielsPoint) -> Self::Output {
        let y_plus_x = &self.y + &self.x;
        let y_minus_x = &self.y - &self.x;
        let pp = &y_plus_x * &rhs.y_plus_x;
        let mm = &y_minus_x * &rhs.y_minus_x;
        let tt2d = &self.t * &rhs.t2d;
        let zz = &self.z * &rhs.z;
        let zz2 = &zz + &zz;
        CompletedPoint {
            x: &pp - &mm,
            y: &pp + &mm,
            z: &zz2 + &tt2d,
            t: &zz2 - &tt2d,
        }
    }
}

impl<'b> Sub<&'b ProjectiveNielsPoint> for &EdwardsPoint {
    type Output = CompletedPoint;

    fn sub(self, rhs: &'b ProjectiveNielsPoint) -> Self::Output {
        let y_plus_x = &self.y + &self.x;
        let y_minus_x = &self.y - &self.x;
        let pm = &y_plus_x * &rhs.y_minus_x;
        let mp = &y_minus_x * &rhs.y_plus_x;
        let tt2d = &self.t * &rhs.t2d;
        let zz = &self.z * &rhs.z;
        let zz2 = &zz + &zz;
        CompletedPoint {
            x: &pm - &mp,
            y: &pm + &mp,
            z: &zz2 - &tt2d,
            t: &zz2 + &tt2d,
        }
    }
}

impl<'b> Add<&'b AffineNielsPoint> for &EdwardsPoint {
    type Output = CompletedPoint;

    fn add(self, rhs: &'b AffineNielsPoint) -> Self::Output {
        let y_plus_x = &self.y + &self.x;
        let y_minus_x = &self.y - &self.x;
        let pp = &y_plus_x * &rhs.y_plus_x;
        let mm = &y_minus_x * &rhs.y_minus_x;
        let txy2d = &self.t * &rhs.xy2d;
        let z2 = &self.z + &self.z;
        CompletedPoint {
            x: &pp - &mm,
            y: &pp + &mm,
            z: &z2 + &txy2d,
            t: &z2 - &txy2d,
        }
    }
}

impl<'b> Sub<&'b AffineNielsPoint> for &EdwardsPoint {
    type Output = CompletedPoint;

    fn sub(self, rhs: &'b AffineNielsPoint) -> Self::Output {
        let y_plus_x = &self.y + &self.x;
        let y_minus_x = &self.y - &self.x;
        let pm = &y_plus_x * &rhs.y_minus_x;
        let mp = &y_minus_x * &rhs.y_plus_x;
        let txy2d = &self.t * &rhs.xy2d;
        let z2 = &self.z + &self.z;
        CompletedPoint {
            x: &pm - &mp,
            y: &pm + &mp,
            z: &z2 - &txy2d,
            t: &z2 + &txy2d,
        }
    }
}
