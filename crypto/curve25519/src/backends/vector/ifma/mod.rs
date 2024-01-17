use crate::{window::NafLookupTable5, EdwardsPoint};
use core::ops::{Add, Neg, Sub};

mod field;
use field::{F51x4Reduced, F51x4Unreduced, Lanes, Shuffle};

pub mod consts;

#[derive(Debug, Clone, Copy)]
pub struct ExtendedPoint(pub(super) F51x4Unreduced);

impl ExtendedPoint {
    pub fn double(&self) -> Self {
        let mut tmp0 = self.0.shuffle(Shuffle::BADC);
        let mut tmp1 = (self.0 + tmp0).shuffle(Shuffle::ABAB);
        tmp0 = self.0.blend(&tmp1, Lanes::D);
        tmp1 = F51x4Reduced::from(tmp0).square();
        let zero = F51x4Unreduced::ZERO;
        let s1_s1_s1_s1 = tmp1.shuffle(Shuffle::AAAA);
        let s2_s2_s2_s2 = tmp1.shuffle(Shuffle::BBBB);
        let s2_s2_s2_s4 = s2_s2_s2_s2.blend(&tmp1, Lanes::D).negate_lazy();
        tmp0 = s1_s1_s1_s1 + zero.blend(&(tmp1 + tmp1), Lanes::C);
        tmp0 = tmp0 + zero.blend(&s2_s2_s2_s2, Lanes::AD);
        tmp0 = tmp0 + zero.blend(&s2_s2_s2_s4, Lanes::BCD);
        let tmp2 = F51x4Reduced::from(tmp0);
        Self(&tmp2.shuffle(Shuffle::DBBD) * &tmp2.shuffle(Shuffle::CACA))
    }
}

#[macros::target_feature("avx512ifma,avx512vl")]
impl Default for ExtendedPoint {
    fn default() -> Self {
        consts::EXTENDEDPOINT_IDENTITY
    }
}

#[macros::target_feature("avx512ifma,avx512vl")]
impl From<EdwardsPoint> for ExtendedPoint {
    fn from(p: EdwardsPoint) -> Self {
        Self(F51x4Unreduced::new(&p.x, &p.y, &p.z, &p.t))
    }
}

#[macros::target_feature("avx512ifma,avx512vl")]
impl From<ExtendedPoint> for EdwardsPoint {
    fn from(p: ExtendedPoint) -> Self {
        let reduced = F51x4Reduced::from(p.0);
        let tmp = F51x4Unreduced::from(reduced).split();
        Self {
            x: tmp[0],
            y: tmp[1],
            z: tmp[2],
            t: tmp[3],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CachedPoint(pub(super) F51x4Reduced);

#[macros::target_feature("avx512ifma,avx512vl")]
impl From<ExtendedPoint> for CachedPoint {
    fn from(p: ExtendedPoint) -> Self {
        let mut x = p.0;
        x = x.blend(&x.diff_sum(), Lanes::AB);
        x = &F51x4Reduced::from(x) * (121_666, 121_666, 2 * 121_666, 2 * 121_665);
        x = x.blend(&x.negate_lazy(), Lanes::D);
        Self(F51x4Reduced::from(x))
    }
}

#[macros::target_feature("avx512ifma,avx512vl")]
impl<'a> Neg for &'a CachedPoint {
    type Output = CachedPoint;

    fn neg(self) -> CachedPoint {
        let swapped = self.0.shuffle(Shuffle::BACD);
        CachedPoint(swapped.blend(&(-self.0), Lanes::D))
    }
}

#[macros::target_feature("avx512ifma,avx512vl")]
impl<'a, 'b> Add<&'b CachedPoint> for &'a ExtendedPoint {
    type Output = ExtendedPoint;

    fn add(self, rhs: &'b CachedPoint) -> ExtendedPoint {
        let mut tmp = self.0;
        tmp = tmp.blend(&tmp.diff_sum(), Lanes::AB);
        tmp = &F51x4Reduced::from(tmp) * &rhs.0;
        tmp = tmp.shuffle(Shuffle::ABDC);
        let tmp = F51x4Reduced::from(tmp.diff_sum());
        let t0 = tmp.shuffle(Shuffle::ADDA);
        let t1 = tmp.shuffle(Shuffle::CBCB);
        ExtendedPoint(&t0 * &t1)
    }
}

#[macros::target_feature("avx512ifma,avx512vl")]
impl<'a, 'b> Sub<&'b CachedPoint> for &'a ExtendedPoint {
    type Output = ExtendedPoint;

    fn sub(self, rhs: &'b CachedPoint) -> ExtendedPoint {
        self + &(-rhs)
    }
}

#[macros::target_feature("avx512ifma,avx512vl")]
impl<'a> From<&'a EdwardsPoint> for NafLookupTable5<CachedPoint> {
    fn from(point: &'a EdwardsPoint) -> Self {
        let a = ExtendedPoint::from(*point);
        let mut a_i = [CachedPoint::from(a); 8];
        let a_2 = a.double();
        for i in 0..7 {
            a_i[i + 1] = (&a_2 + &a_i[i]).into();
        }
        Self(a_i)
    }
}
