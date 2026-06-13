#!/usr/bin/env bash
# Report active vs ignored conformance test counts for CI transparency.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CONF="$ROOT/crates/ontologos-conformance/tests"

count_in_dir() {
  local pattern="$1"
  local dir="$2"
  if [[ ! -d "$dir" ]]; then
    echo 0
    return
  fi
  rg -c "$pattern" "$dir"/*.rs 2>/dev/null | awk -F: '{s+=$2} END {print s+0}'
}

TOTAL=$(count_in_dir '#\[test\]' "$CONF")
IGNORED=$(count_in_dir '#\[ignore' "$CONF")
ACTIVE=$((TOTAL - IGNORED))

echo "ontologos-conformance test inventory"
echo "  total test functions: $TOTAL"
echo "  ignored (dormant):    $IGNORED"
echo "  active in default CI: $ACTIVE"
echo ""
echo "Note: default 'cargo test' skips #[ignore] tests."
echo "Run ignored tier: cargo test -p ontologos-conformance -- --ignored"

if (( ACTIVE < 1 )); then
  echo "error: expected at least one active conformance test" >&2
  exit 1
fi
