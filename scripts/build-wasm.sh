#!/bin/sh
set -eu

cd "$(dirname "$0")/.."

out=src/lib/encre-wasm
sysroot="$(rustc --print sysroot)"
commit="$(rustc -vV | sed -n 's/^commit-hash: //p')"
cargo_home="${CARGO_HOME:-$HOME/.cargo}"
remap="--remap-path-prefix=$sysroot/lib/rustlib/src/rust=/rustc/$commit --remap-path-prefix=$cargo_home=/cargo"
CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS="${CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS:--Ccodegen-units=1 $remap}" wasm-pack build crates/encre-wasm --release --target web --no-pack --out-dir "../../$out" --out-name encre_wasm
rm -f "$out/.gitignore"
