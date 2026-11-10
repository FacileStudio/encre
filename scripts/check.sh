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
  go_check
  filet check
}

go_check() {
  unformatted="$(cd go && gofmt -l .)"
  if [ -n "$unformatted" ]; then
    echo "gofmt: $unformatted" >&2
    return 1
  fi
  (cd go && go vet ./...)
}

test_suites() {
  cargo test --all-features --locked
  cargo test --locked -p encre --test hostile -- --ignored --test-threads=1
  bun run test
  cargo build --locked -p encre-cli
  (cd go && ENCRE_BIN="$(pwd)/../target/debug/encre" go test -count=1 ./...)
}

case "$mode" in
  check) check ;;
  test) test_suites ;;
  all) check && test_suites ;;
  *) echo "usage: scripts/check.sh [check|test|all]" >&2; exit 2 ;;
esac
