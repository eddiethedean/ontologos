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
  rg -c '#\[ignore' "$file" || echo 0
}

HERMIT_IGNORED=$(count_ignores "$GENERATED")
WG_IGNORED=$(count_ignores "$WG_GENERATED")
TOTAL_IGNORED=$((HERMIT_IGNORED + WG_IGNORED))

echo "HermiT catalog ignored: $HERMIT_IGNORED"
echo "OWL WG catalog ignored: $WG_IGNORED"
echo "Total ignored: $TOTAL_IGNORED"

# Phase 0 gate: runnable subset must pass (non-ignored tests only; fast clausify batch).
cargo test -p ontologos-conformance --test hermit_generated -- --test-threads=4

BUDGET_FILE="$ROOT/benchmarks/data/hermit/catalog/ignore_budget.txt"
if [[ -f "$BUDGET_FILE" ]]; then
  BUDGET=$(cat "$BUDGET_FILE")
  if (( TOTAL_IGNORED > BUDGET )); then
    echo "error: ignore budget exceeded ($TOTAL_IGNORED > $BUDGET)" >&2
    exit 1
  fi
  echo "ignore budget OK ($TOTAL_IGNORED <= $BUDGET)"
fi
