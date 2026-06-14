#!/usr/bin/env bash
# Report DL OFN fixture semantic pass rate by HermiT Java class family.
# Informational — does not fail CI when pass rate is low.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT}"

BIN="$("${ROOT}/benchmarks/scripts/build-conformance-tools.sh")"
"${BIN}/dl_ofn_pass_rate"
