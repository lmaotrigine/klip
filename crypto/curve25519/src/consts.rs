#[cfg(curve25519_bits = "32")]
pub use crate::backends::serial::u32::consts::*;
#[cfg(curve25519_bits = "64")]
pub use crate::backends::serial::u64::consts::*;
