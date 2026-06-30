#!/usr/bin/env bash
# Track shrinking #[ignore] budget for HermiT generated conformance tests.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
GENERATED="$ROOT/crates/ontologos-conformance/tests/hermit_generated.rs"
WG_GENERATED="$ROOT/crates/ontologos-conformance/tests/hermit_wg_generated.rs"

count_ignores() {
  local file="$1"
  if [[ ! -f "$file" ]]; then
    echo 0
    return
  fi
  local count=0
  if command -v rg >/dev/null 2>&1; then
    count=$(rg -c '#\[ignore' "$file" 2>/dev/null | head -1) || count=0
  else
    count=$(grep -c '#\[ignore' "$file" 2>/dev/null) || count=0
  fi
  echo "${count:-0}"
}

HERMIT_IGNORED=$(count_ignores "$GENERATED")
WG_IGNORED=$(count_ignores "$WG_GENERATED")
TOTAL_IGNORED=$((HERMIT_IGNORED + WG_IGNORED))

echo "HermiT catalog ignored: $HERMIT_IGNORED"
echo "OWL WG catalog ignored: $WG_IGNORED"
echo "Total ignored: $TOTAL_IGNORED"

# Runnable subset is exercised by the CI "HermiT conformance (Tier A)" step.
BUDGET_FILE="$ROOT/benchmarks/data/hermit/catalog/ignore_budget.txt"
if [[ -f "$BUDGET_FILE" ]]; then
  BUDGET=$(cat "$BUDGET_FILE")
  if (( TOTAL_IGNORED > BUDGET )); then
    echo "error: ignore budget exceeded ($TOTAL_IGNORED > $BUDGET)" >&2
    exit 1
  fi
  echo "ignore budget OK ($TOTAL_IGNORED <= $BUDGET)"
fi
