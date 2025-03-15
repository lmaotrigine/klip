use super::{Backend, Bits};

fn warn_default(msg: &str) {
    println!("cargo::warning=\"defaulting to curve25519_bits=32: {msg}\"");
}

pub fn determine_bits() -> Bits {
    match std::env::var("CARGO_CFG_TARGET_POINTER_WIDTH").as_deref() {
        Ok("64") => Bits::SixtyFour,
        Ok("32") => Bits::ThirtyTwo,
        Ok(_) => {
            warn_default("target pointer width is neither 32 nor 64.");
            Bits::ThirtyTwo
        }
        Err(_) => {
            warn_default("standard CARGO_CFG_TARGET_POINTER_WIDTH environment variable is not set");
            Bits::ThirtyTwo
        }
    }
}

pub fn determine_backend(target_arch: &str, bits: Bits) -> Backend {
    match std::env::var("CARGO_CFG_CURVE25519_BACKEND").as_deref() {
        Ok("serial") => Backend::Serial,
        Ok("simd") => {
            if is_simd_capable(target_arch, bits) {
                Backend::Simd
            } else {
                panic!(
                    "could not override curve25519_backend to simd: simd support is not available \
                     for this target."
                )
            }
        }
        _ => {
            if is_simd_capable(target_arch, bits) {
                Backend::Simd
            } else {
                Backend::Serial
            }
        }
    }
}

fn is_simd_capable(arch: &str, bits: Bits) -> bool {
    arch == "x86_64" && bits == Bits::SixtyFour
}

pub fn is_nightly() -> bool {
    macro_rules! t {
        ($e:expr) => {
            match $e {
                Some(e) => e,
                None => panic!("failed to get rustc version"),
            }
        };
    }
    let rustc = t!(std::env::var_os("RUSTC"));
    let output = t!(std::process::Command::new(rustc)
        .arg("--version")
        .output()
        .ok());
    assert!(
        output.status.success(),
        "failed to run rustc: {}",
        String::from_utf8_lossy(output.stderr.as_slice())
    );
    let version = t!(std::str::from_utf8(&output.stdout).ok());
    let mut pieces = version.split('.');
    assert!(
        pieces.next() == Some("rustc 1"),
        "failed to get rustc version"
    );
    let nightly_raw = t!(pieces.nth(1)).split('-').nth(1);
    nightly_raw.is_some_and(|raw| raw.starts_with("dev") || raw.starts_with("nightly"))
}
