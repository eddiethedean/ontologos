#!/usr/bin/env bash
# Verify ROADMAP 1.0.0 exit criteria. Fails until all gates are green — do not tag 1.0.0 before this passes.
# Release tagging and publish remain DEFERRED until this script exits 0.
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

# Workspace must be 1.0.0 once release gates pass.
check "workspace version is 1.0.0" grep -q 'version = "1.0.0"' "${ROOT}/Cargo.toml"

# Conformance active test budget (target ≥400 at 1.0).
ACTIVE="$("${ROOT}/benchmarks/scripts/report-conformance-coverage.sh" 2>/dev/null | awk '/active in default CI/ {print $NF}')"
if [[ "${ACTIVE:-0}" -ge 400 ]]; then
  echo "OK  active conformance tests (${ACTIVE} ≥ 400)"
else
  echo "FAIL active conformance tests (${ACTIVE:-0} < 400 target)" >&2
  FAIL=1
fi

# Tier A: promoted HermiT lists @ 30s + phase closures (full suite gate pending 17 axiom fixes).
check "Tier A promoted conformance" bash -c "
  set -euo pipefail
  export ONTOLOGOS_DL_BUDGET_SECS=30
  \"${ROOT}/benchmarks/scripts/hermit-burndown.sh\" test
  cargo test -p ontologos-conformance --release --quiet --locked \\
    --test phase3_closure --test phase4_closure --test phase8_closure --test phase9_closure
"

# Tier B/C harness.
check "Tier B classification gate script" test -x "${ROOT}/benchmarks/scripts/compare-classification-fixtures.sh"
check "Tier B classification fixtures" "${ROOT}/benchmarks/scripts/compare-classification-fixtures.sh"
check "Tier C PR gate script" test -x "${ROOT}/benchmarks/scripts/compare-tier-c-gate.sh"
check "Tier C smoke script" test -x "${ROOT}/benchmarks/scripts/compare-hermit-tier-c.sh"
check "DL taxonomy gate script" test -x "${ROOT}/benchmarks/scripts/compare-dl-taxonomy.sh"
check "HermiT JAR download script" test -x "${ROOT}/benchmarks/scripts/download-hermit-jar.sh"
check "reference baseline script" test -x "${ROOT}/benchmarks/scripts/run-reference-baseline.sh"
check "Tier C PR gate" "${ROOT}/benchmarks/scripts/compare-tier-c-gate.sh"
check "Tier C harness" "${ROOT}/benchmarks/scripts/compare-hermit-tier-c.sh"

if [[ "${FAIL}" -ne 0 ]]; then
  echo "" >&2
  echo "1.0.0 release gates not met — see ROADMAP.md and docs/migration/v0.9.x-to-v1.0.0.md" >&2
  exit 1
fi

echo "All 1.0.0 release gates satisfied."
