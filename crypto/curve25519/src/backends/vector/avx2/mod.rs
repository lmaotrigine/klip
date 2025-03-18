pub mod consts;
mod field;
use core::ops::{Add, Neg, Sub};

use crypto_common::constant_time::{Choice, ConditionallySelectable};
use field::{FieldElement2625x4, Lanes, Shuffle};

use crate::window::NafLookupTable5;

#[derive(Debug, Clone, Copy)]
pub struct ExtendedPoint(pub(super) FieldElement2625x4);

#[macros::target_feature("avx2")]
impl ExtendedPoint {
    pub fn double(&self) -> Self {
        let mut tmp0 = self.0.shuffle(Shuffle::ABAB);
        let mut tmp1 = tmp0.shuffle(Shuffle::BADC);
        tmp0 = self.0.blend(tmp0 + tmp1, Lanes::D);
        tmp1 = tmp0.square_and_negate_d();
        let zero = FieldElement2625x4::ZERO;
        let s_1 = tmp1.shuffle(Shuffle::AAAA);
        let s_2 = tmp1.shuffle(Shuffle::BBBB);
        tmp0 = zero.blend(tmp1 + tmp1, Lanes::C);
        tmp0 = tmp0.blend(tmp1, Lanes::D);
        tmp0 = tmp0 + s_1;
        tmp0 = tmp0 + zero.blend(s_2, Lanes::AD);
        tmp0 = tmp0 + zero.blend(s_2.negate_lazy(), Lanes::BC);
        tmp1 = tmp0.shuffle(Shuffle::DBBD);
        tmp0 = tmp0.shuffle(Shuffle::CACA);
        Self(&tmp0 * &tmp1)
    }
}

#[macros::target_feature("avx2")]
impl From<crate::EdwardsPoint> for ExtendedPoint {
    fn from(p: crate::EdwardsPoint) -> Self {
        Self(FieldElement2625x4::new(&p.x, &p.y, &p.z, &p.t))
    }
}

#[macros::target_feature("avx2")]
impl From<ExtendedPoint> for crate::EdwardsPoint {
    fn from(p: ExtendedPoint) -> Self {
        let tmp = p.0.split();
        Self {
            x: tmp[0],
            y: tmp[1],
            z: tmp[2],
            t: tmp[3],
        }
    }
}

#[macros::target_feature("avx2")]
impl ConditionallySelectable for ExtendedPoint {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Self(FieldElement2625x4::conditional_select(&a.0, &b.0, choice))
    }

    fn conditional_assign(&mut self, other: &Self, choice: Choice) {
        self.0.conditional_assign(&other.0, choice);
    }
}

#[macros::target_feature("avx2")]
impl Default for ExtendedPoint {
    fn default() -> Self {
        consts::EXTENDEDPOINT_IDENTITY
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CachedPoint(pub(super) FieldElement2625x4);

#[macros::target_feature("avx2")]
impl From<ExtendedPoint> for CachedPoint {
    fn from(p: ExtendedPoint) -> Self {
        let mut x = p.0;
        x = x.blend(x.diff_sum(), Lanes::AB);
        x = x * (121_666, 121_666, 2 * 121_666, 2 * 121_665);
        x = x.blend(-x, Lanes::D);
        Self(x)
    }
}

#[macros::target_feature("avx2")]
impl Neg for &CachedPoint {
    type Output = CachedPoint;

    fn neg(self) -> CachedPoint {
        let swapped = self.0.shuffle(Shuffle::BACD);
        CachedPoint(swapped.blend(swapped.negate_lazy(), Lanes::D))
    }
}

#[macros::target_feature("avx2")]
impl Add<&CachedPoint> for &ExtendedPoint {
    type Output = ExtendedPoint;

    fn add(self, rhs: &CachedPoint) -> ExtendedPoint {
        let mut tmp = self.0;
        tmp = tmp.blend(tmp.diff_sum(), Lanes::AB);
        tmp = &tmp * &rhs.0;
        tmp = tmp.shuffle(Shuffle::ABDC);
        tmp = tmp.diff_sum();
        let t0 = tmp.shuffle(Shuffle::ADDA);
        let t1 = tmp.shuffle(Shuffle::CBCB);
        ExtendedPoint(&t0 * &t1)
    }
}

#[macros::target_feature("avx2")]
impl Sub<&CachedPoint> for &ExtendedPoint {
    type Output = ExtendedPoint;

    fn sub(self, rhs: &CachedPoint) -> ExtendedPoint {
        self + &(-rhs)
    }
}

#[macros::target_feature("avx2")]
impl From<&crate::EdwardsPoint> for NafLookupTable5<CachedPoint> {
    fn from(value: &crate::EdwardsPoint) -> Self {
        let a = ExtendedPoint::from(*value);
        let mut a_i = [CachedPoint::from(a); 8];
        let a_2 = a.double();
        for i in 0..7 {
            a_i[i + 1] = (&a_2 + &a_i[i]).into();
        }
        Self(a_i)
    }
}
