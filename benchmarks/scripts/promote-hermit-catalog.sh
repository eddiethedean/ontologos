#!/usr/bin/env bash
# Scan planned HermiT cases, promote passing ones, regenerate catalog artifacts.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

echo "==> Scanning planned axiom cases for promotion (release build)"
BIN="$("${ROOT}/benchmarks/scripts/build-conformance-tools.sh")"
"${BIN}/promote_catalog"

echo "==> Regenerating HermiT catalog"
python3 tests/hermit/generate_catalog.py --promote-only 2>/dev/null || python3 tests/hermit/generate_catalog.py
python3 tests/hermit/generate_catalog.py --wg-catalog-only 2>/dev/null || true
echo "==> Scanning WG cases for promotion (release build)"
"${BIN}/promote_wg"
python3 tests/hermit/generate_catalog.py --promote-wg-only 2>/dev/null || true

echo "==> Updating ignore budget"
HERMIT_IGNORED=$(grep -c '#\[ignore' crates/ontologos-conformance/tests/hermit_generated.rs || echo 0)
WG_IGNORED=$(grep -c '#\[ignore' crates/ontologos-conformance/tests/hermit_wg_generated.rs 2>/dev/null || echo 0)
echo $((HERMIT_IGNORED + WG_IGNORED)) > benchmarks/data/hermit/catalog/ignore_budget.txt

echo "==> Done. Run conformance tests to verify promoted cases."
