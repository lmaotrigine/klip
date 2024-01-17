#[macros::target_feature_specialize("avx2", conditional("avx512ifma,avx512vl", nightly))]
pub mod spec {
    use crate::{window::NafLookupTable5, EdwardsPoint, Scalar};
    use core::cmp::Ordering;

    #[for_target_feature("avx2")]
    use crate::backends::vector::avx2::{
        consts::BASEPOINT_ODD_LOOKUP_TABLE, CachedPoint, ExtendedPoint,
    };
    #[for_target_feature("avx512ifma")]
    use crate::backends::vector::ifma::{
        consts::BASEPOINT_ODD_LOOKUP_TABLE, CachedPoint, ExtendedPoint,
    };

    #[allow(clippy::cast_sign_loss)]
    pub fn mul(a: &Scalar, big_a: &EdwardsPoint, b: &Scalar) -> EdwardsPoint {
        let a_naf = a.non_adjacent_form(5);
        let b_naf = b.non_adjacent_form(8);
        let mut i = 255;
        for j in (0..256).rev() {
            i = j;
            if a_naf[i] != 0 || b_naf[i] != 0 {
                break;
            }
        }
        let table_a = NafLookupTable5::<CachedPoint>::from(big_a);
        let table_b = &BASEPOINT_ODD_LOOKUP_TABLE;
        let mut q = ExtendedPoint::default();
        loop {
            q = q.double();
            match a_naf[i].cmp(&0) {
                Ordering::Greater => {
                    q = &q + &table_a.select(a_naf[i] as usize);
                }
                Ordering::Less => {
                    q = &q - &table_a.select(-a_naf[i] as usize);
                }
                Ordering::Equal => {}
            }
            match b_naf[i].cmp(&0) {
                Ordering::Greater => {
                    q = &q + &table_b.select(b_naf[i] as usize);
                }
                Ordering::Less => {
                    q = &q - &table_b.select(-b_naf[i] as usize);
                }
                Ordering::Equal => {}
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }
        q.into()
    }
}
