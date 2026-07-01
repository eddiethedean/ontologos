#!/usr/bin/env bash
# PR CI gates — user-facing contract and corpus scripts (not full HermiT parity).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAIL=0

check() {
  local name="$1"
  shift
  if "$@"; then
    echo "OK  ${name}"
  else
    echo "FAIL ${name}" >&2
    FAIL=1
  fi
}

CONTRACT_TESTS="$("${ROOT}/benchmarks/scripts/report-conformance-coverage.sh" 2>/dev/null | awk '/contract test functions/ {print $NF}')"
CASE_IDS_FILE="${ROOT}/crates/ontologos-contract/data/case_ids.txt"
CASE_IDS=0
if [[ -f "${CASE_IDS_FILE}" ]]; then
  CASE_IDS=$(grep -cve '^\s*$' -e '^\s*#' "${CASE_IDS_FILE}" || true)
fi
CONTRACT_COVERAGE=$((CONTRACT_TESTS + CASE_IDS))
if [[ "${CONTRACT_COVERAGE}" -ge 50 ]]; then
  echo "OK  contract coverage (${CONTRACT_COVERAGE} = ${CONTRACT_TESTS} tests + ${CASE_IDS} catalog cases, min 50)"
else
  echo "FAIL contract coverage (${CONTRACT_COVERAGE} below 50 target)" >&2
  FAIL=1
fi

check "pizza EL golden script" test -x "${ROOT}/benchmarks/scripts/compare-pizza-el-golden.sh"
check "pizza EL golden" "${ROOT}/benchmarks/scripts/compare-pizza-el-golden.sh"
check "Tier B classification fixtures" "${ROOT}/benchmarks/scripts/compare-classification-fixtures.sh"

if [[ "${FAIL}" -ne 0 ]]; then
  echo "" >&2
  echo "PR gates not met — see docs/reference/conformance.md" >&2
  exit 1
fi

echo "All PR gates satisfied."
