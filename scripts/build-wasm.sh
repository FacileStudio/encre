#!/bin/sh
set -eu

cd "$(dirname "$0")/.."

out=src/lib/encre-wasm
CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS="${CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS:--Ccodegen-units=1}" wasm-pack build crates/encre-wasm --release --target web --no-pack --out-dir "../../$out" --out-name encre_wasm
rm -f "$out/.gitignore"
