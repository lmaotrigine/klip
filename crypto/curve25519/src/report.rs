#[cfg(curve25519_backend = "serial")]
compile_error!("curve25519_backend is 'serial'");

#[cfg(curve25519_backend = "simd")]
compile_error!("curve25519_backend is 'simd'");

#[cfg(curve25519_bits = "32")]
compile_error!("curve25519_bits is '32'");

#[cfg(curve25519_bits = "64")]
compile_error!("curve25519_bits is '64'");
