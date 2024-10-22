#!/usr/bin/env -S just --justfile

# vim: ft=make ts=2 sts=2 et

ci := env_var_or_default("CI", "")
release := env_var_or_default("RELEASE", "")
use-cross := env_var_or_default("USE_CROSS", "")
extra-build-args := env_var_or_default("EXTRA_BUILD_ARGS", "")
extra-features := env_var_or_default("EXTRA_FEATURES", "")
default-features := env_var_or_default("DEFAULT_FEATURES", "")
override-features := env_var_or_default("OVERRIDE_FEATURES", "")
timings := env_var_or_default("TIMINGS", "")
build-std := env_var_or_default("BUILD_STD", "")

cargo := if use-cross != "" { "cross" } else { "cargo" }
export CARGO := cargo

host := `rustc -vV | grep host: | cut -d ' ' -f2`
target := env_var_or_default("CARGO_BUILD_TARGET", host)
target-os := if target =~ "-windows-" {
  "windows"
} else if target =~ "-darwin" {
  "macos"
} else if target =~ "-linux-" {
  "linux"
} else if target =~ "bsd" {
  "bsd"
} else if target =~ "dragonfly" {
  "bsd"
} else {
  "unknown"
}
target-arch := if target =~ "x86_64" {
  "x64"
} else if target =~ "i[56]86" {
  "x86"
} else if target =~ "aarch64" {
  "arm64"
} else if target =~ "armv7" {
  "arm32"
} else {
  "unknown"
}
target-libc := if target =~ "-gnu" {
  "gnu"
} else if target =~ "-musl" {
  "musl"
} else {
  "unknown"
}
output-ext := if target-os == "windows" { ".exe" } else { "" }
output-filename := "klip" + output-ext
output-profile-dir := if release != "" { "release" } else { "debug" }
output-dir := "target" / target / output-profile-dir
output-path := output-dir / output-filename

cargo-profile := if release != "" { "release" } else { "dev" }
is-ci := if ci != "" { "y" } else { "n" }
cargo-buildstd := if build-std != "" {
  " -Zbuild-std=std,panic_abort -Zbuild-std-features=panic_immediate_abort"
} else if target == "x86_64h-apple-darwin" {
  " -Zbuild-std=std,panic_abort -Zbuild-std-features=panic_immediate_abort"
} else {
  ""
}
rustc-gcclibs := if (cargo-profile / is-ci / target-libc) == "release/y/musl" {
  " -C link-arg=-lgcc -C link-arg=-static-libgcc"
} else {
  ""
}
cargo-no-default-features := if default-features == "false" {
  " --no-default-features"
} else if default-features == "true" {
  ""
} else if (cargo-profile / is-ci) == "dev/y" {
  " --no-default-features"
} else {
  ""
}

default-cargo-features := if cargo-buildstd != "" {
  ""
} else {
  ""
}

cargo-features := trim_end_match(
  default-cargo-features +
  if override-features != "" {
    override-features
  } else if (cargo-profile / is-ci) == "dev/y" {
    ""
  } else if (cargo-profile / is-ci) == "release/y" {
    ""
  } else if extra-features != "" {
    extra-features
  } else {
    ""
  },
  ","
)
cargo-split-debuginfo := if cargo-buildstd != "" { " --config='profile.release.split-debuginfo=\"packed\"' --config=profile.release.debug=1" } else { "" }
rustc-icf := if release != "" {
  if target-os == "windows" {
    " -C link-arg=-Wl,--icf=safe"
  } else if target-os == "linux" {
    " -C link-arg=-Wl,--icf=safe"
  } else {
    ""
  }
} else {
  ""
}

share-generics := if cargo-buildstd != "" {
  " -Zshare-generics=y"
} else {
  ""
}

link-args := if target-os == "windows" {
  " -C target-feature=+crt-static"
} else if target =~ "-musl" {
  " -C target-feature=+crt-static -C link-self-contained=yes -C link-arg=-fuse-ld=lld -C link-arg=-Wl,--compress-debug-sections=zlib -C linker=clang"
} else if target-os != "macos" {
  " -C link-arg=-fuse-ld=lld -C link-arg=-Wl,--compress-debug-sections=zlib -C linker=clang"
} else {
  " -C link-arg=-Wl,--compress-debug-sections=zlib -C linker=clang"
}

cargo-check-args := (" --target ") + (target) + (cargo-buildstd) + (if extra-build-args != "" { " " + extra-build-args } else { "" }) + (cargo-split-debuginfo)
cargo-build-args :=  " --locked " + (if release != "" { "--release" } else { "" }) + (cargo-check-args) + (cargo-no-default-features) + (if cargo-features != "" { " --features " + cargo-features } else { "" }) + (if timings != "" { "--timings" } else { "" })
export RUSTFLAGS := (rustc-gcclibs) + (rustc-icf) + (link-args) + (share-generics) + " -C symbol-mangling-version=v0 -C force-frame-pointers=yes" + (if ci == "" { " -C target-cpu=native" } else { "" })

toolchain-name := if cargo-buildstd != "" { "nightly" } else { "stable" }
target-name := if target == "x86_64h-apple-darwin" { "" } else { target }
default-components := if cargo-buildstd != "" { "rust-src" } else { "" }

_default:
  @just --list

toolchain components=default-components:
  rustup toolchain install {{toolchain-name}} {{ if components != "" { "--component " + components } else { "" } }} --no-self-update --profile minimal {{ if target-name != "" { "--target " + target-name } else { "" } }}
  rustup override set {{toolchain-name}}

print-env:
  @echo "env RUSTFLAGS='$RUSTFLAGS', CARGO='$CARGO'"

print-rustflags:
  @echo "$RUSTFLAGS"

build: print-env
  {{cargo}} build {{cargo-build-args}}

deb: print-env
  cargo deb --profile deb --locked --target {{target}}

check: print-env
  {{cargo}} check {{cargo-build-args}}
  cargo hack check --feature-powerset {{cargo-check-args}}

get-output file outdir=".":
  test -d "{{outdir}}" || mkdir -p {{outdir}}
  cp -r {{ output-dir / file }} {{outdir}}/{{ file_name(file) }}
  -ls -l {{outdir}}/{{ file_name(file) }}

get-binary outdir=".": (get-output output-filename outdir)
  -chmod +x {{ outdir / output-filename }}

unit-tests: print-env
  {{cargo}} test {{cargo-build-args}}

clippy: print-env
  cargo hack --feature-powerset clippy -- -D warnings

fmt check="": print-env
  cargo fmt --all -- {{check}}

fmt-check: (fmt "--check")

lint: clippy fmt-check

package-dir:
  rm -rf packages/prep
  mkdir -p packages/prep/doc
  cp LICENSE packages/prep
  cp README.md packages/prep
  cp -r completions packages/prep
  cp doc/klip.1 packages/prep/doc

[macos]
package-prepare: build package-dir
  just get-binary packages/prep
  -just get-output klip.dSYM packages/prep

[linux]
package-prepare: build package-dir
  just get-binary packages/prep
  -cp {{output-dir}}/deps/klip-*.dwp packages/prep/klip.dwp

[windows]
package-prepare: build package-dir
  just get-binary packages/prep
  -just get-output deps/klip.pdb packages/prep

[macos]
lipo-prepare: package-dir
  just target=aarch64-apple-darwin build get-binary packages/prep/arm64
  just target=x86_64-apple-darwin build get-binary packages/prep/x64
  just target=x86_64h-apple-darwin build get-binary packages/prep/x64h

  just target=aarch64-apple-darwin get-binary packages/prep/arm64
  just target=x86_64-apple-darwin get-binary packages/prep/x64
  just target=x86_64h-apple-darwin get-binary packages/prep/x64h
  lipo -create -output packages/prep/{{output-filename}} packages/prep/{arm64,x64,x64h}/{{output-filename}}

  rm -rf packages/prep/{arm64,x64,x64h}

[linux]
package: package-prepare
  cd packages/prep && tar cv {{output-filename}} | xz -9 > "../klip-{{target}}.tar.xz"
  cd packages/prep && tar cv * | xz -9 > "../klip-{{target}}.full.tar.xz"
  cd packages && shasum -a 256 "klip-{{target}}.tar.xz" > "klip-{{target}}.tar.xz.sha256"
  cd packages && shasum -a 256 "klip-{{target}}.full.tar.xz" > "klip-{{target}}.full.tar.xz.sha256"

[macos]
package: package-prepare
  cd packages/prep && zip -r -9 "../klip-{{target}}.zip" {{output-filename}}
  cd packages/prep && zip -r -9 "../klip-{{target}}.full.zip" *
  cd packages && shasum -a 256 "klip-{{target}}.zip" > "klip-{{target}}.zip.sha256"
  cd packages && shasum -a 256 "klip-{{target}}.full.zip" > "klip-{{target}}.full.zip.sha256"

[windows]
package: package-prepare
  cd packages/prep && 7z a -mx9 "../klip-{{target}}.zip" {{output-filename}}
  cd packages/prep && 7z a -mx9 "../kli-{{target}}.full.zip" *
  cd packages && certutil -hashfile "klip-{{target}}.zip" SHA256 > "klip-{{target}}.zip.sha256"
  cd packages && certutil -hashfile "klip-{{target}}.full.zip" SHA256 > "klip-{{target}}.full.zip.sha256"

[macos]
package-lipo: lipo-prepare
  cd packages/prep && zip -r -9 "../klip-universal-apple-darwin.zip" {{output-filename}}
  cd packages/prep && zip -r -9 "../klip-universal-apple-darwin.full.zip" *
  cd packages && shasum -a 256 "klip-universal-apple-darwin.zip" > "klip-universal-apple-darwin.zip.sha256"
  cd packages && shasum -a 256 "klip-universal-apple-darwin.full.zip" > "klip-universal-apple-darwin.full.zip.sha256"

[macos]
repackage-lipo: package-dir
  set -euxo pipefail
  mkdir -p packages/prep/{arm64,x64,x64h}
  cd packages/prep/x64 && unzip -o "../../klip-x86_64-apple-darwin.full.zip"
  cd packages/prep/x64h && unzip -o "../../klip-x86_64h-apple-darwin.full.zip"
  cd packages/prep/arm64 && unzip -o "../../klip-aarch64-apple-darwin.full.zip"
  lipo -create -output packages/prep/{{output-filename}} packages/prep/{arm64,x64,x64h}/{{output-file}}
  ./packages/prep/{{output-filename}} --version
  rm -rf packages/prep/{arm64,x64,x64h}
  cd packages/prep && zip -r -9 "../klip-universal-apple-darwin.zip" {{output-filename}}
  cd packages/prep && zip -r -9 "../klip-universal-apple-darwin.full.zip" *
  cd packages && shasum -a 256 "klip-universal-apple-darwin.zip" > "klip-universal-apple-darwin.zip.sha256"
  cd packages && shasum -a 256 "klip-universal-apple-darwin.full.zip" > "klip-universal-apple-darwin.full.zip.sha256"
