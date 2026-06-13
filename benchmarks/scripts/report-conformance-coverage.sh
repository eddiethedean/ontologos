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
  local total=0
  local count
  for file in "$dir"/*.rs; do
    [[ -f "$file" ]] || continue
    count=$(grep -cF "$pattern" "$file" 2>/dev/null || true)
    total=$((total + count))
  done
  echo "$total"
}

TOTAL=$(count_in_dir '#[test]' "$CONF")
IGNORED=$(count_in_dir '#[ignore' "$CONF")
ACTIVE=$((TOTAL - IGNORED))

CATALOG="${ROOT}/benchmarks/data/hermit/catalog/cases.json"

echo "ontologos-conformance test inventory"
echo "  total test functions: $TOTAL"
echo "  ignored (dormant):    $IGNORED"
echo "  active in default CI: $ACTIVE"
echo ""

if [[ -f "$CATALOG" ]]; then
  python3 - "$CATALOG" <<'PY'
import json, sys
from collections import Counter
cases = json.load(open(sys.argv[1]))
by_status = Counter(c["status"] for c in cases)
by_engine = Counter(c["engine"] for c in cases)
axiom = [c for c in cases if c["status"] == "axiom"]
print("HermiT catalog (cases.json)")
print(f"  total cases: {len(cases)}")
for status in sorted(by_status):
    print(f"  status {status}: {by_status[status]}")
print("  by engine:", ", ".join(f"{k}={v}" for k, v in sorted(by_engine.items())))
print(f"  axiom cases (runnable): {len(axiom)}")
PY
  echo ""
fi

echo "Note: default 'cargo test' skips #[ignore] tests."
echo "Run ignored tier: cargo test -p ontologos-conformance -- --ignored"

if (( ACTIVE < 1 )); then
  echo "error: expected at least one active conformance test" >&2
  exit 1
fi
