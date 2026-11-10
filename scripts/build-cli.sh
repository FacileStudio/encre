#!/bin/sh
set -eu

cd "$(dirname "$0")/.."

out="${ENCRE_CLI_OUT:-target/cli}"
targets="${ENCRE_CLI_TARGETS:-x86_64-unknown-linux-musl aarch64-unknown-linux-musl}"
host="$(rustc -vV | sed -n 's/^host: //p')"
lld="$(rustc --print sysroot)/lib/rustlib/$host/bin/rust-lld"
version="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)"

arch_of() {
  case "$1" in
    x86_64-*) echo amd64 ;;
    aarch64-*) echo arm64 ;;
    *) echo "build-cli: no Go arch name for $1" >&2; exit 2 ;;
  esac
}

c_compiler() {
  arch="${1%%-*}"
  for cc in "$arch-linux-musl-gcc" "$arch-linux-gnu-gcc"; do
    if command -v "$cc" >/dev/null 2>&1; then echo "$cc"; return; fi
  done
  case "$host" in
    "$arch"-*) command -v gcc >/dev/null 2>&1 && echo gcc ;;
  esac
}

rm -rf "$out"
mkdir -p "$out"
failed=""
for target in $targets; do
  echo "▸ Building encre $version for $target"
  cc="$(c_compiler "$target" || true)"
  if [ -z "$cc" ]; then
    echo "✗ no C compiler for $target; typst's psm crate needs one, install gcc-${target%%-*}-linux-gnu" >&2
    failed="$failed $target"
    continue
  fi
  if env "CC_$(echo "$target" | tr - _)=$cc" CARGO_PROFILE_RELEASE_STRIP=symbols \
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER="$lld" \
    cargo build --release --locked -p encre-cli --target "$target"; then
    cp "target/$target/release/encre" "$out/encre-$target"
    tar -C "target/$target/release" -czf "$out/encre_${version}_linux_$(arch_of "$target").tar.gz" encre
  else
    echo "✗ encre did not build for $target" >&2
    failed="$failed $target"
  fi
done

if [ -n "$(ls "$out")" ]; then
  (cd "$out" && sha256sum ./*.tar.gz | sed 's| \./| |' > checksums.txt)
fi
ls -l "$out"
if [ -n "$failed" ]; then
  echo "✗ failed targets:$failed" >&2
  exit 1
fi
