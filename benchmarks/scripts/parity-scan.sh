#!/usr/bin/env bash
# Run all HermiT parity scan tools (release build, parallel case evaluation).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT}"
BIN="$("${ROOT}/benchmarks/scripts/build-conformance-tools.sh")"

echo "==> Planned DL failures"
"${BIN}/dl_failures"
echo
echo "==> Promotable axiom cases"
"${BIN}/promote_catalog"
echo
echo "==> DL OFN pass rate"
"${BIN}/dl_ofn_pass_rate"
