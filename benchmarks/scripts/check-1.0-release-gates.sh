#!/usr/bin/env bash
# Verify 1.0.x release exit criteria. Fails until all gates are green — do not tag a 1.0.x release before this passes.
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

# Workspace must match the staged 1.0.x release version.
check "workspace version is 1.0.1" grep -q 'version = "1.0.1"' "${ROOT}/Cargo.toml"

# Conformance active test budget (target ≥400 at 1.0; nightly/release only).
ACTIVE="$("${ROOT}/benchmarks/scripts/report-conformance-coverage.sh" 2>/dev/null | awk '/active parity \(nightly\)/ {print $NF}')"
if [[ "${ACTIVE:-0}" -ge 400 ]]; then
  echo "OK  active parity tests (${ACTIVE} ≥ 400)"
else
  echo "FAIL active parity tests (${ACTIVE:-0} < 400 target)" >&2
  FAIL=1
fi

# Tier A: full HermiT + OWL WG catalog @ 30s + phase closures (Phase 9).
check "Tier A full conformance" bash -c "
  set -euo pipefail
  export ONTOLOGOS_DL_BUDGET_SECS=30
  unset ONTOLOGOS_CI_PROMOTED_ONLY
  cargo test -p ontologos-conformance --release --quiet --locked \\
    --test hermit_generated \\
    --test hermit_rdfs \\
    --test hermit_rl \\
    --test hermit_el \\
    --test hermit_parser \\
    --test hermit_wg_generated \\
    -- \\
    --test-threads=4 \\
    --skip planned_engine_failure_scan \\
    --skip ian_backjumping3_axiom_check_completes_within_budget
  cargo test -p ontologos-conformance --release --quiet --locked \\
    --test phase3_closure --test phase4_closure --test phase8_closure --test phase9_closure \\
    -- --skip phase9_true_parity_pct_is_100
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
  echo "1.0.x release gates not met — see ROADMAP.md and docs/migration/v0.9.x-to-v1.0.0.md" >&2
  exit 1
fi

echo "All 1.0.x release gates satisfied."
