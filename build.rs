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

fn emit_rerun_if_git_changed() {
    // this dance is required to actually trigger a rebuild on every commit
    // .git/HEAD only changes when you switch branches or go to a detached head
    // so we read the ref it points to and emit a rerun-if-changed for that as
    // well
    let head = std::path::Path::new(".git/HEAD");
    if !head.exists() {
        return;
    }
    println!("cargo::rerun-if-changed=.git/HEAD");
    let Ok(contents) = std::fs::read_to_string(head) else { return };
    let Some(ref_) = contents.strip_prefix("ref: ") else { return };
    let ref_file = std::path::Path::new(".git").join(ref_.trim());
    if ref_file.exists() {
        println!("cargo::rerun-if-changed={}", ref_file.display());
    }
}

fn set_git_hash() {
    use std::process::Command;
    if let Ok(hash) = std::env::var("KLIP_BUILD_GIT_HASH") {
        println!("cargo::rustc-env=KLIP_BUILD_GIT_HASH={hash}");
        return;
    }
    let args = &["rev-parse", "--short", "HEAD"];
    match Command::new("git").args(args).output() {
        Ok(output) => {
            let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if hash.is_empty() {
                println!(
                    "cargo::warning=output from `git rev-parse` is empty, so skipping embedding \
                     of commit hash"
                );
                return;
            }
            println!("cargo::rustc-env=KLIP_BUILD_GIT_HASH={hash}");
        }
        Err(e) => {
            println!(
                "cargo::warning=failed to run `git rev-parse`, so skipping embedding of commit \
                 hash: {e}"
            );
        }
    }
}

fn main() {
    const MANIFEST: &str = "pkg/windows/Manifest.xml";
    println!("cargo::rerun-if-changed={MANIFEST}");
    println!("cargo::rerun-if-env-changed=KLIP_BUILD_GIT_HASH");
    emit_rerun_if_git_changed();
    set_git_hash();
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=CARGO_BUILD_TARGET");
    let target_os = std::env::var("CARGO_CFG_TARGET_OS");
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV");
    if !(target_os.as_deref() == Ok("windows") && target_env.as_deref() == Ok("msvc")) {
        return;
    }
    // embed the Windows application manifest file
    let Ok(mut manifest) = std::env::current_dir() else { return };
    manifest.push(MANIFEST);
    let Ok(_) = manifest.canonicalize() else { return };
    let Some(manifest) = manifest.to_str() else { return };
    println!("cargo::rustc-link-arg-bin=klip=/MANIFEST:EMBED");
    println!("cargo::rustc-link-arg-bin=klip=/MANIFESTINPUT:{manifest}");
    // delay-load userenv.dll so that it, and the rpcrt4.dll and ucrtbase.dll it
    // pulls in, aren't loaded at startup. `std::env::home_dir` only calls into
    // it when `USERPROFILE` isn't set, which is quite uncommon. This is kept
    // here rather than in .cargo/config.toml so that it applies when doing
    // `cargo install` or overriding `RUSTFLAGS`
    println!("cargo::rustc-link-arg-bin=klip=/DELAYLOAD:userenv.dll");
    println!("cargo::rustc-link-arg-bin=klip=delayimp.lib");
    // turn linker warnings into errors. this helps debugging on older
    // compilers, and also we do want to bail if embedding the manifest fails
    println!("cargo::rustc-link-arg-bin=klip=/WX");
}
