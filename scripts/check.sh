#!/bin/sh
set -eu

cd "$(dirname "$0")/.."

mode="${1:-all}"

check() {
  cargo fmt --check
  cargo clippy --all-targets --all-features --locked -- -D warnings
  bun install --frozen-lockfile
  bun run check
  sh scripts/build-wasm.sh
  git diff --exit-code -- src/lib/encre-wasm
  filet check
}

test_suites() {
  cargo test --all-features --locked
  bun run test
}

case "$mode" in
  check) check ;;
  test) test_suites ;;
  all) check && test_suites ;;
  *) echo "usage: scripts/check.sh [check|test|all]" >&2; exit 2 ;;
esac
