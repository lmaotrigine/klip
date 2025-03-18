use crate::{EdwardsPoint, Scalar};

pub mod serial;
#[cfg(curve25519_backend = "simd")]
pub mod vector;

#[derive(Clone, Copy)]
enum Backend {
    #[cfg(curve25519_backend = "simd")]
    Avx2,
    #[cfg(all(curve25519_backend = "simd", nightly))]
    Avx512,
    Serial,
}

#[inline]
#[allow(clippy::missing_const_for_fn)]
fn get_selected_backend() -> Backend {
    #[cfg(all(curve25519_backend = "simd", nightly))]
    {
        target_features::detect!(avx512, "avx512ifma", "avx512vl");
        let tok = avx512::init();
        if tok.get() {
            return Backend::Avx512;
        }
    }
    #[cfg(curve25519_backend = "simd")]
    {
        target_features::detect!(avx2, "avx2");
        let tok = avx2::init();
        if tok.get() {
            return Backend::Avx2;
        }
    }
    Backend::Serial
}

pub fn vartime_double_base_mul(a: &Scalar, big_a: &EdwardsPoint, b: &Scalar) -> EdwardsPoint {
    match get_selected_backend() {
        #[cfg(curve25519_backend = "simd")]
        Backend::Avx2 => vector::scalar_mul::vartime_double_base::spec_avx2::mul(a, big_a, b),
        #[cfg(all(curve25519_backend = "simd", nightly))]
        Backend::Avx512 => {
            vector::scalar_mul::vartime_double_base::spec_avx512ifma_avx512vl::mul(a, big_a, b)
        }
        Backend::Serial => serial::scalar_mul::vartime_double_base::mul(a, big_a, b),
    }
}
