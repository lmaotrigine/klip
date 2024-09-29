#![forbid(
    dead_code,
    deprecated,
    future_incompatible,
    missing_copy_implementations,
    missing_debug_implementations,
    nonstandard_style,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unused,
    unsafe_code,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used
)]

fn set_git_hash() {
    use std::process::Command;
    let args = &["rev-parse", "--short", "HEAD"];
    let Ok(output) = Command::new("git").args(args).output() else {
        return;
    };
    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if hash.is_empty() {
        return;
    }
    println!("cargo:rustc-env=KLIP_BUILD_GIT_HASH={hash}");
}

fn main() {
    const MANIFEST: &str = "../pkg/windows/Manifest.xml";
    set_git_hash();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=TARGET");
    let target_os = std::env::var("CARGO_CFG_TARGET_OS");
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV");
    if !(target_os.as_deref() == Ok("windows") && target_env.as_deref() == Ok("msvc")) {
        return;
    }
    println!("cargo:rerun-if-changed={MANIFEST}");
    let Ok(mut manifest) = std::env::current_dir() else {
        return;
    };
    manifest.push(MANIFEST);
    let Some(manifest) = manifest.to_str() else {
        return;
    };
    println!("cargo:rustc-link-arg-bin=klip=/DEPENDENTLOADFLAG:0x800");
    println!("cargo:rustc-link-arg-bin=klip=/DELAYLOAD:bcrypt.dll");
    println!("cargo:rustc-link-arg-bin=klip=delayimp.lib");
    println!("cargo:rustc-link-arg-bin=klip=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg-bin=klip=/MANIFESTINPUT:{manifest}");
    println!("cargo:rustc-link-arg-bin=klip=/WX");
}
