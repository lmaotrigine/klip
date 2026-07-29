#!/usr/bin/env -S just --justfile

set lazy

toolchain := `rustc -vV | head -n 1 | cut -d ' ' -f 2`
rustflags := if toolchain =~ "-nightly" {
  "-C target-cpu=native -Z unstable-options -C panic=immediate-abort"
} else {
  "-C target-cpu=native"
}
buildflags := if toolchain =~ "-nightly" {
  "-Zbuild-std=std,panic_abort"
} else {
  ""
}

_default:
  @just --list

alias c := check

tag := `git rev-parse --short HEAD`
image := "ghcr.io/lmaotrigine/klip"
release := `git describe --tags --exact-match 2>/dev/null || true`

# run clippy
[group('lint')]
[env('RUSTFLAGS', '-Wunused-crate-dependencies')]
check:
  cargo clippy --workspace --all-targets

# run cargo fmt
[group('lint')]
fmt *args="":
  cargo fmt {{args}}

# check for trailing whitespace and carriage returns
[group('lint')]
ws:
  ! rg '\s+$'
  ! rg '\r'

# perform all linting tasks
[group('lint')]
lint: check (fmt "--all --check") shellcheck ws

# jemalloc uses some intrinsics that are not implemented natively by rust yet.
# so we use zigbuild here because we build the stdlib.
# this reduces the binary size by ~200KiB compared to linking against GCC.
# see: https://github.com/rust-lang/rust/issues/46651#issuecomment-1847872105

# build a non-portable release binary with host CPU features.
[group('build')]
build-release-native target="x86_64-unknown-linux-musl":
  RUSTFLAGS="{{rustflags}}" cargo zigbuild {{buildflags}} --release --target {{target}}

# build a docker image from the current source tree.
[group('build')]
docker *args="":
  TAG="{{tag}}" IMAGE_NAME="{{image}}" RELEASE="{{release}}" docker buildx bake {{args}}

_assert_tag_at_head:
  @git describe --tags --exact-match HEAD 2>&1 > /dev/null

# update the man page with current release version and date.
[group('release')]
update-man: _assert_tag_at_head
  perl -i -pe 's/[0-9]\+\.[0-9]\+\.[0-9]\+/{{release}}/g' doc/klip.1
  perl -i -pe "s/[0-9]{4}-[0-9]{2}-[0-9]{2}/$(date -u +%Y-%m-%d)/g" doc/klip.1

# run shellcheck on all shell scripts
[group('lint')]
shellcheck:
  shellcheck ci/* scripts/* pkg/debian/*
  fd -e sh -tf -x shellcheck

# bump package version and create a new git tag.
[group('release')]
bump version="":
  scripts/bump {{version}}
