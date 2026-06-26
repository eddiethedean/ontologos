#!/usr/bin/env bash
# Scan planned HermiT cases, promote passing ones, regenerate catalog artifacts.
#
# Default: incremental (unpromoted cases only). Use --full for complete rescan.
# Prefer: bash benchmarks/scripts/hermit-burndown.sh promote
#   ONTOLOGOS_DL_BUDGET_SECS=30     — fast scan; use 120 for final promotion
#   ONTOLOGOS_DL_MAX_WORKERS=10     — concurrent DL ops (default 10)
#   ONTOLOGOS_SCAN_THREADS=10       — rayon case parallelism (default 10)
#   ONTOLOGOS_TABLEAU_MAX_STALL_STEPS=4096 — large nominal WG fixtures
#
# Incremental mode skips re-checking catalog cases already at status=axiom|wg.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

# shellcheck source=burndown-process-guard.sh
source "${ROOT}/benchmarks/scripts/burndown-process-guard.sh"
burndown_guard_begin

BIN="$("${ROOT}/benchmarks/scripts/build-conformance-tools.sh")"
export ONTOLOGOS_DL_BUDGET_SECS="${ONTOLOGOS_DL_BUDGET_SECS:-120}"
export ONTOLOGOS_DL_MAX_WORKERS="${ONTOLOGOS_DL_MAX_WORKERS:-10}"
export ONTOLOGOS_SCAN_THREADS="${ONTOLOGOS_SCAN_THREADS:-10}"

echo "==> Scanning planned axiom cases for promotion (release, incremental)"
"${BIN}/promote_catalog" --incremental

echo "==> Applying axiom promotion to catalog"
python3 tests/hermit/generate_catalog.py --promote-only

echo "==> Scanning WG cases for promotion (release, incremental)"
"${BIN}/promote_wg" --incremental

echo "==> Applying WG promotion to catalog"
python3 tests/hermit/generate_catalog.py --promote-wg-only

echo "==> Updating ignore budget"
HERMIT_IGNORED=$(grep -c '#\[ignore' crates/ontologos-conformance/tests/hermit_generated.rs || echo 0)
WG_IGNORED=$(grep -c '#\[ignore' crates/ontologos-conformance/tests/hermit_wg_generated.rs 2>/dev/null || echo 0)
echo $((HERMIT_IGNORED + WG_IGNORED)) > benchmarks/data/hermit/catalog/ignore_budget.txt

echo "==> Done. Run conformance tests to verify promoted cases."
