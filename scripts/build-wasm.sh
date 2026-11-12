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

glue="$out/encre_wasm.js"
for line in 'let wasmModule, wasmInstance, wasm;' 'let __wbg_init_promise;' 'let cachedUint8ArrayMemory0 = null;'; do
  if ! grep -qxF "$line" "$glue"; then
    echo "build-wasm: the wasm-bindgen glue no longer declares '$line'; update __encre_reset" >&2
    exit 1
  fi
done
cat >> "$glue" <<'EOF'

/**
 * Drops the current instance, so the next `initSync` or `init` builds a fresh one after a trap.
 * @returns {WebAssembly.Module | undefined} the compiled module, to hand to `initSync`
 */
export function __encre_reset() {
    const module = wasmModule;
    wasm = undefined;
    wasmInstance = undefined;
    wasmModule = undefined;
    __wbg_init_promise = undefined;
    cachedUint8ArrayMemory0 = null;
    return module;
}
EOF
cat >> "$out/encre_wasm.d.ts" <<'EOF'

/** Drops the current instance, so the next `initSync` or `init` builds a fresh one after a trap; returns its compiled module. */
export function __encre_reset(): WebAssembly.Module | undefined;
EOF
