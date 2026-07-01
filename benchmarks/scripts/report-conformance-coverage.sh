#!/usr/bin/env bash
# Report contract vs parity test counts for CI transparency.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CONF="$ROOT/crates/ontologos-conformance/tests"
CONTRACT="$ROOT/crates/ontologos-contract/tests"

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

CONTRACT_TOTAL=$(count_in_dir '#[test]' "$CONTRACT")
PARITY_TOTAL=$(count_in_dir '#[test]' "$CONF")
PARITY_IGNORED=$(count_in_dir '#[ignore' "$CONF")
PARITY_ACTIVE=$((PARITY_TOTAL - PARITY_IGNORED))

CATALOG="${ROOT}/benchmarks/data/hermit/catalog/cases.json"

echo "ontologos-contract test inventory (PR gate)"
echo "  contract test functions: $CONTRACT_TOTAL"
echo ""
echo "ontologos-conformance test inventory (nightly parity)"
echo "  total test functions: $PARITY_TOTAL"
echo "  ignored (dormant):    $PARITY_IGNORED"
echo "  active parity (nightly): $PARITY_ACTIVE"
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

echo "Note: PR CI runs cargo test -p ontologos-contract --release"
echo "Note: parity tier runs cargo test -p ontologos-conformance (nightly / release)"

if (( CONTRACT_TOTAL < 1 )); then
  echo "error: expected at least one contract test" >&2
  exit 1
fi

if (( PARITY_ACTIVE < 1 )); then
  echo "error: expected at least one active parity test" >&2
  exit 1
fi
