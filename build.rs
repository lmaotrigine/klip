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
    if let Ok(hash) = std::env::var("KLIP_BUILD_GIT_HASH") {
        if hash == "skip" {
            println!("cargo::rustc-env=KLIP_BUILD_GIT_HASH=");
        } else {
            println!("cargo::rustc-env=KLIP_BUILD_GIT_HASH= (rev {hash})");
        }
        return;
    }
    let args = &["rev-parse", "--short", "HEAD"];
    let Ok(output) = Command::new("git").args(args).output() else {
        return;
    };
    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if hash.is_empty() {
        return;
    }
    println!("cargo::rustc-env=KLIP_BUILD_GIT_HASH= (rev {hash})");
}

fn main() {
    const MANIFEST: &str = "pkg/windows/Manifest.xml";
    println!("cargo::rerun-if-changed={MANIFEST}");
    println!("cargo::rerun-if-env-changed=KLIP_BUILD_GIT_HASH");
    set_git_hash();
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=CARGO_BUILD_TARGET");
    let target_os = std::env::var("CARGO_CFG_TARGET_OS");
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV");
    if !(target_os.as_deref() == Ok("windows") && target_env.as_deref() == Ok("msvc")) {
        return;
    }
    // embed the Windows application manifest file
    let Ok(mut manifest) = std::env::current_dir() else {
        return;
    };
    manifest.push(MANIFEST);
    let Ok(_) = manifest.canonicalize() else {
        return;
    };
    let Some(manifest) = manifest.to_str() else {
        return;
    };
    println!("cargo::rustc-link-arg-bin=klip=/MANIFEST:EMBED");
    println!("cargo::rustc-link-arg-bin=klip=/MANIFESTINPUT:{manifest}");
    // only search system32 for DLLs
    //
    // this applies to DLLs loaded at load time.
    // this setting is ignored on Windows versions prior to Windows 10 RS1
    // (1601).
    // https://learn.microsoft.com/en-us/cpp/build/reference/dependentloadflag?view=msvc-170
    println!("cargo::rustc-link-arg-bin=klip=/DEPENDENTLOADFLAG:0x800");
    // delay load
    //
    // delay load DLLs that are not "known DLLs"[1].
    // known DLLs are always loaded from the system directory whereas other
    // DLLs are loaded from the application directory.
    // by delay loading the latter, we can ensure they are instead loaded from
    // the system directory.
    // [1]: https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order#factors-that-affect-searching
    //
    // this will work on all supported Windows versions but it relies on us
    // using SetDefaultDllDirectories before any libraries are loaded.
    // see also: lib/platform/preflight/windows.rs
    let delay_load_dlls = ["bcrypt", "api-ms-win-core-synch-l1-2-0"];
    for dll in delay_load_dlls {
        println!("cargo::rustc-link-arg-bin=klip=/DELAYLOAD:{dll}.dll");
    }
    // when using delayload, it's necessary to also link delayimp.lib
    // https://learn.microsoft.com/en-us/cpp/build/reference/dependentloadflag?view=msvc-170
    println!("cargo::rustc-link-arg-bin=klip=delayimp.lib");
    // turn linker warnings into errors
    //
    // rust hides linker warnings, meaning mistakes may go unnoticed.
    // turning them into errors forces them to be displayed (and the build to
    // fail).
    // if we do want to ignore specific warnings, then /IGNORE: should be used.
    println!("cargo::rustc-link-arg-bin=klip=/WX");
}
