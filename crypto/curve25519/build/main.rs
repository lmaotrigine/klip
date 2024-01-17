#![deny(
    dead_code,
    deprecated,
    future_incompatible,
    missing_copy_implementations,
    missing_debug_implementations,
    nonstandard_style,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used
)]

mod deterministic;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Bits {
    SixtyFour = 64,
    ThirtyTwo = 32,
}

impl std::fmt::Display for Bits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", *self as u8)
    }
}

#[derive(Clone, Copy)]
enum Backend {
    Simd,
    Serial,
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serial => write!(f, "\"serial\""),
            Self::Simd => write!(f, "\"simd\""),
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build/main.rs");
    println!("cargo:rerun-if-changed=build/deterministic.rs");
    println!("cargo:rerun-if-env-changed=TARGET");
    let bits = match std::env::var("CARGO_CFG_CURVE25519_BITS").as_deref() {
        Ok("32") => Bits::ThirtyTwo,
        Ok("64") => Bits::SixtyFour,
        _ => deterministic::determine_bits(),
    };
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let nightly = deterministic::is_nightly();
    let backend = deterministic::determine_backend(&target_arch, bits);
    println!("cargo:rustc-cfg=curve25519_bits={bits}");
    println!("cargo:rustc-cfg=curve25519_backend={backend}");
    if nightly {
        println!("cargo:rustc-cfg=nightly");
    }
}
