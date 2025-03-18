use crate::{
    backends::serial::curve_models::{AffineNielsPoint, ProjectiveNielsPoint},
    EdwardsPoint,
};
use core::fmt::Debug;
use crypto_common::{
    constant_time::{Choice, ConditionallyNegatable, ConditionallySelectable, ConstantTimeEq},
    erase::Erase,
};

#[derive(Clone, Copy)]
pub struct LookupTable<T: Default + ConditionallySelectable + ConditionallyNegatable>(
    pub(crate) [T; 8],
);

impl<T: Default + ConditionallySelectable + ConditionallyNegatable> LookupTable<T> {
    #[allow(
        clippy::cast_lossless,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    pub fn select(&self, x: i8) -> T {
        debug_assert!(x >= (-8));
        debug_assert!(x <= 8);
        let xmask = x as i16 >> 7;
        let xabs = (x as i16 + xmask) ^ xmask;
        let mut t = T::default();
        for j in 1..9 {
            let c = (xabs as u16).ct_eq(&(j as u16));
            t.conditional_assign(&self.0[j - 1], c);
        }
        let neg_mask = Choice::from((xmask & 1) as u8);
        t.conditional_negate(neg_mask);
        t
    }
}
impl<T: Copy + Default + ConditionallySelectable + ConditionallyNegatable> Default
    for LookupTable<T>
{
    fn default() -> Self {
        Self([T::default(); 8])
    }
}
impl<T: Debug + Default + ConditionallySelectable + ConditionallyNegatable> Debug
    for LookupTable<T>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "LookupTable({:?})", &self.0)
    }
}
impl<T: Copy + Default + ConditionallySelectable + ConditionallyNegatable + Erase> Erase
    for LookupTable<T>
{
    fn erase(&mut self) {
        self.0.iter_mut().erase();
    }
}

impl<'a> From<&'a EdwardsPoint> for LookupTable<AffineNielsPoint> {
    fn from(p: &'a crate::EdwardsPoint) -> Self {
        let mut points = [p.as_affine_niels(); 8];
        for j in 0..7 {
            points[j + 1] = (p + &points[j]).as_extended().as_affine_niels();
        }
        Self(points)
    }
}

pub type LookupTableRadix16<T> = LookupTable<T>;

#[derive(Clone, Copy)]
pub struct NafLookupTable5<T: Debug + Copy>(pub(crate) [T; 8]);

impl<T: Debug + Copy> NafLookupTable5<T> {
    pub fn select(&self, x: usize) -> T {
        debug_assert_eq!(x & 1, 1);
        debug_assert!(x < 16);
        self.0[x / 2]
    }
}

impl<T: Debug + Copy> Debug for NafLookupTable5<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "NaflLookupTable5({:?})", &self.0)
    }
}

impl<'a> From<&'a EdwardsPoint> for NafLookupTable5<ProjectiveNielsPoint> {
    fn from(a: &'a EdwardsPoint) -> Self {
        let mut a_i = [a.as_projective_niels(); 8];
        let a2 = a.double();
        for i in 0..7 {
            a_i[i + 1] = (&a2 + &a_i[i]).as_extended().as_projective_niels();
        }
        Self(a_i)
    }
}

#[derive(Clone, Copy)]
pub struct NafLookupTable8<T: Debug + Copy>(pub(crate) [T; 64]);

impl<T: Debug + Copy> NafLookupTable8<T> {
    pub fn select(&self, x: usize) -> T {
        debug_assert_eq!(x & 1, 1);
        debug_assert!(x < 128);
        self.0[x / 2]
    }
}

impl<T: Debug + Copy> Debug for NafLookupTable8<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "NafLookupTable8([")?;
        for i in 0..64 {
            writeln!(f, "\t{:?},", self.0[i])?;
        }
        write!(f, "])")
    }
}
