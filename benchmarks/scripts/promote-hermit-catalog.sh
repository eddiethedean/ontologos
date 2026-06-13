#!/usr/bin/env bash
# Scan planned HermiT cases, promote passing ones, regenerate catalog artifacts.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

echo "==> Scanning planned axiom cases for promotion"
cargo run -q -p ontologos-conformance --bin promote_catalog

echo "==> Regenerating HermiT catalog"
python3 tests/hermit/generate_catalog.py

echo "==> Updating ignore budget"
HERMIT_IGNORED=$(rg -c '#\[ignore' crates/ontologos-conformance/tests/hermit_generated.rs || echo 0)
WG_IGNORED=$(rg -c '#\[ignore' crates/ontologos-conformance/tests/hermit_wg_generated.rs 2>/dev/null || echo 0)
echo $((HERMIT_IGNORED + WG_IGNORED)) > benchmarks/data/hermit/catalog/ignore_budget.txt

echo "==> Done. Run conformance tests to verify promoted cases."
