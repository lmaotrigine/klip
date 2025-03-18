use crate::{
    backends::serial::curve_models::{ProjectiveNielsPoint, ProjectivePoint},
    window::NafLookupTable5,
    EdwardsPoint, Scalar,
};
use core::cmp::Ordering;

#[allow(clippy::many_single_char_names, clippy::cast_sign_loss)]
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
    let table_a = NafLookupTable5::<ProjectiveNielsPoint>::from(big_a);
    let table_b = &crate::consts::AFFINE_ODD_MULTIPLES_OF_BASEPOINT;
    let mut r = ProjectivePoint::default();
    loop {
        let mut t = r.double();
        match a_naf[i].cmp(&0) {
            Ordering::Greater => t = &t.as_extended() + &table_a.select(a_naf[i] as usize),
            Ordering::Less => t = &t.as_extended() - &table_a.select(-a_naf[i] as usize),
            Ordering::Equal => {}
        }
        match b_naf[i].cmp(&0) {
            Ordering::Greater => t = &t.as_extended() + &table_b.select(b_naf[i] as usize),
            Ordering::Less => t = &t.as_extended() - &table_b.select(-b_naf[i] as usize),
            Ordering::Equal => {}
        }
        r = t.as_projective();
        if i == 0 {
            break;
        }
        i -= 1;
    }
    r.as_extended()
}
