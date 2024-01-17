#!/bin/sh

match_and_report() {
  _pattern=$1
  _file=$2
  if grep -q "$_pattern" "$_file"; then
    echo build OK "$_file" : "$_pattern"
  else
    echo build ERROR "$_file" : "$_pattern"
    echo ">>>>>>>>>>>>>>>>>>>>>>>>>>"
    cat "$_file"
    echo "<<<<<<<<<<<<<<<<<<<<<<<<<<"
    exit 1
  fi
}

# all of this assumes 64-bit host
cargo clean
OUT=build_1.txt
RUSTFLAGS="--cfg curve25519_report" cargo build > "$OUT" 2>&1
match_and_report "curve25519_backend is 'simd'" "$OUT"
match_and_report "curve25519_bits is '64'" "$OUT"

cargo clean
OUT=build_2.txt
RUSTFLAGS="--cfg curve25519_report --cfg curve25519_bits=\"32\"" cargo build > "$OUT" 2>&1
match_and_report "curve25519_backend is 'serial'" "$OUT"
match_and_report "curve25519_bits is '32'" "$OUT"

cargo clean
OUT=build_3.txt
RUSTFLAGS="--cfg curve25519_report --cfg curve25519_bits=\"64\"" cargo build --target i686-unknown-linux-gnu > "$OUT" 2>&1
match_and_report "curve25519_backend is 'serial'" "$OUT"
match_and_report "curve25519_bits is '64'" "$OUT"

cargo clean
OUT=build_4.txt
RUSTFLAGS="--cfg curve25519_report" cargo build --target i686-unknown-linux-gnu > "$OUT" 2>&1
match_and_report "curve25519_backend is 'serial'" "$OUT"
match_and_report "curve25519_bits is '32'" "$OUT"

cargo clean
OUT=build_5.txt
RUSTFLAGS="--cfg curve25519_report --cfg curve25519_backend=\"serial\"" cargo build > "$OUT" 2>&1
match_and_report "curve25519_backend is 'serial'" "$OUT"
match_and_report "curve25519_bits is '64'" "$OUT"

cargo clean
OUT=build_6.txt
RUSTFLAGS="--cfg curve25519_report --cfg curve25519_backend=\"serial\" --cfg curve25519_bits=\"32\"" cargo build > "$OUT" 2>&1
match_and_report "curve25519_backend is 'serial'" "$OUT"
match_and_report "curve25519_bits is '32'" "$OUT"
