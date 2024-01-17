pub mod avx2;
#[cfg(nightly)]
pub mod ifma;
pub mod scalar_mul;
#[allow(clippy::use_self)]
pub mod simd;
