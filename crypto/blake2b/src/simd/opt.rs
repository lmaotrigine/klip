#[cfg(feature = "simd_opt")]
use super::transmute_shuffle;
use super::u64x4;
#[cfg(feature = "simd_opt")]
use super::{u16x16, u32x8, u8x32};

#[cfg(feature = "simd_opt")]
#[inline(always)]
pub fn rotate_right(vec: u64x4, n: u32) -> u64x4 {
    match n {
        32 => rotate_right_32(vec),
        24 => rotate_right_24(vec),
        16 => rotate_right_16(vec),
        _ => _rotate_right(vec, n),
    }
}

#[cfg(all(feature = "simd", not(feature = "simd_opt")))]
#[inline(always)]
pub fn rotate_right(vec: u64x4, n: u32) -> u64x4 {
    _rotate_right(vec, n)
}

#[cfg(not(feature = "simd"))]
#[inline(always)]
pub const fn rotate_right(vec: u64x4, n: u32) -> u64x4 {
    u64x4::new(
        vec.0.rotate_right(n),
        vec.1.rotate_right(n),
        vec.2.rotate_right(n),
        vec.3.rotate_right(n),
    )
}

#[inline(always)]
fn _rotate_right(vec: u64x4, n: u32) -> u64x4 {
    let r = u64::from(n);
    let l = 64 - r;
    (vec >> u64x4::new(r, r, r, r)) ^ (vec << u64x4::new(l, l, l, l))
}

#[cfg(feature = "simd_opt")]
#[inline(always)]
fn rotate_right_32(vec: u64x4) -> u64x4 {
    if cfg!(any(target_feature = "sse2", target_feature = "neon")) {
        transmute_shuffle!(u32x8, 8, vec, [1, 0, 3, 2, 5, 4, 7, 6])
    } else {
        _rotate_right(vec, 32)
    }
}

#[cfg(feature = "simd_opt")]
#[inline(always)]
fn rotate_right_24(vec: u64x4) -> u64x4 {
    if cfg!(all(
        feature = "simd_asm",
        target_feature = "neon",
        target_arch = "arm"
    )) {
        rotate_right_vext(vec, 3)
    } else if cfg!(target_feature = "ssse3") {
        transmute_shuffle!(
            u8x32,
            32,
            vec,
            [
                3, 4, 5, 6, 7, 0, 1, 2, 11, 12, 13, 14, 15, 8, 9, 10, 19, 20, 21, 22, 23, 16, 17,
                18, 27, 28, 29, 30, 31, 24, 25, 26
            ]
        )
    } else {
        _rotate_right(vec, 24)
    }
}

#[cfg(feature = "simd_opt")]
#[inline(always)]
fn rotate_right_16(vec: u64x4) -> u64x4 {
    if cfg!(all(
        feature = "simd_asm",
        target_feature = "neon",
        target_arch = "arm"
    )) {
        rotate_right_vext(vec, 2)
    } else if cfg!(target_feature = "ssse3") {
        transmute_shuffle!(
            u8x32,
            32,
            vec,
            [
                2, 3, 4, 5, 6, 7, 0, 1, 10, 11, 12, 13, 14, 15, 8, 9, 18, 19, 20, 21, 22, 23, 16,
                17, 26, 27, 28, 29, 30, 31, 24, 25
            ]
        )
    } else if cfg!(target_feature = "sse2") {
        transmute_shuffle!(
            u16x16,
            16,
            vec,
            [1, 2, 3, 0, 5, 6, 7, 4, 9, 10, 11, 8, 13, 14, 15, 12]
        )
    } else {
        _rotate_right(vec, 16)
    }
}

#[cfg(all(feature = "simd_asm", target_feature = "neon", target_arch = "arm"))]
mod neon {
    use crate::simd::{ops::simd_shuffle, u64x4};

    #[allow(non_camel_case_types)]
    #[derive(Debug, Clone, Copy)]
    #[repr(simd)]
    struct u64x2(pub u64, pub u64);

    #[inline(always)]
    unsafe fn vext_u64(vec: u64x2, b: u8) -> u64x2 {
        let result;
        core::arch::asm!("vext.8 ${0:e}, ${1:e}, ${1:e}, $2\nvext.8 ${0:f}, ${1:f}, ${1:f}, $2" : "=w" (result) : "w" (vec), "n" (b));
        result
    }

    #[inline(always)]
    pub fn rotate_right_vext(vec: u64x4, b: u8) -> u64x4 {
        unsafe {
            let tmp0 = vext_u64(simd_shuffle(vec, vec, [0, 1]), b);
            let tmp1 = vext_u64(simd_shuffle(vec, vec, [2, 3]), b);
            simd_shuffle(tmp0, tmp1, [0, 1, 2, 3])
        }
    }
}

#[cfg(all(feature = "simd_asm", target_feature = "neon", target_arch = "arm"))]
use neon::rotate_right_vext;

#[cfg(all(
    feature = "simd_opt",
    not(all(feature = "simd_asm", target_feature = "neon", target_arch = "arm"))
))]
fn rotate_right_vext(_: u64x4, _: u8) -> u64x4 {
    unreachable!()
}
