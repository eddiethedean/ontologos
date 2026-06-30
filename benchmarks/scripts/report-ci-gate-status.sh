#!/usr/bin/env bash
# Print 1.0 release gate status and conformance snapshot for HermiT parity tracking.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT}"

echo "OntoLogos CI gate status ($(date -u +%Y-%m-%dT%H:%MZ))"
echo ""

"${ROOT}/benchmarks/scripts/report-conformance-coverage.sh"
echo ""

BIN="$("${ROOT}/benchmarks/scripts/build-conformance-tools.sh" 2>/dev/null || true)"
if [[ -x "${BIN}/parity_status" ]]; then
  "${BIN}/parity_status"
  echo ""
fi

if bash "${ROOT}/benchmarks/scripts/check-1.0-release-gates.sh" >/tmp/ontologos-gates.log 2>&1; then
  echo "1.0 release gates: PASS"
  grep '^OK' /tmp/ontologos-gates.log || true
else
  echo "1.0 release gates: FAIL"
  cat /tmp/ontologos-gates.log
  exit 1
fi

echo ""
if bash "${ROOT}/benchmarks/scripts/check-hermit-parity-phases.sh" 2>&1; then
  echo "HermiT parity phases: PASS (100% catalog)"
else
  echo "HermiT parity phases: in progress (see ROADMAP.md)"
fi

echo ""
if bash "${ROOT}/benchmarks/scripts/check-true-parity-gate.sh" 2>&1; then
  echo "True parity gate: PASS (blocking @ 100%)"
else
  echo "True parity gate: FAIL (see parity-roadmap.md)"
fi

echo ""
BIN="$("${ROOT}/benchmarks/scripts/build-conformance-tools.sh" 2>/dev/null || true)"
if [[ -x "${BIN}/dl_ofn_pass_rate" ]]; then
  "${BIN}/dl_ofn_pass_rate" | head -8
fi
