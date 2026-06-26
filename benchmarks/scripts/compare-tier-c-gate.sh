#!/usr/bin/env bash
# PR-blocking Tier C gate — vendored OntoLogos DL taxonomy goldens (no JVM).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

echo "HermiT Tier C PR gate (vendored goldens)"

chmod +x "${ROOT}/benchmarks/scripts/compare-dl-taxonomy.sh"
chmod +x "${ROOT}/benchmarks/scripts/compare-pizza-el-golden.sh"

"${ROOT}/benchmarks/scripts/compare-dl-taxonomy.sh"
"${ROOT}/benchmarks/scripts/compare-pizza-el-golden.sh"

echo "Tier C PR gate: all checks passed"
