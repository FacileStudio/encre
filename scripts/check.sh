#!/bin/sh
set -eu

cd "$(dirname "$0")/.."

mode="${1:-all}"

check() {
  bun install --frozen-lockfile
  bun run check
  filet check
}

test_suites() {
  bun run test
}

case "$mode" in
  check) check ;;
  test) test_suites ;;
  all) check && test_suites ;;
  *) echo "usage: scripts/check.sh [check|test|all]" >&2; exit 2 ;;
esac
