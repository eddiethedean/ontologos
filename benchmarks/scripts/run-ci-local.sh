#!/usr/bin/env bash
# Mirror .github/workflows/ci.yml locally (main + python + docs; skip windows-only jobs).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

export CARGO_TERM_COLOR=always
export CARGO_INCREMENTAL=0
export ONTOLOGOS_DL_BUDGET_SECS=30

LOG="${ROOT}/target/ci-local.log"
mkdir -p "${ROOT}/target"
: >"$LOG"

step() {
  local name="$1"
  shift
  echo "========== ${name} ==========" | tee -a "$LOG"
  if "$@" >>"$LOG" 2>&1; then
    echo "PASS: ${name}" | tee -a "$LOG"
  else
    echo "FAIL: ${name} (log: ${LOG})" | tee -a "$LOG"
    exit 1
  fi
}

step_optional() {
  local name="$1"
  shift
  echo "========== ${name} (optional) ==========" | tee -a "$LOG"
  if "$@" >>"$LOG" 2>&1; then
    echo "PASS: ${name}" | tee -a "$LOG"
  else
    echo "SKIP/FAIL: ${name} (log: ${LOG})" | tee -a "$LOG"
  fi
}

echo "OntoLogos local CI — $(date -u +%Y-%m-%dT%H:%M:%SZ)" | tee -a "$LOG"

# --- job: check ---
step download ./benchmarks/scripts/download.sh
step fmt cargo fmt --all -- --check
step clippy cargo clippy --workspace --all-targets -- -D warnings
step test-workspace cargo test --workspace --exclude ontologos-conformance --locked
step hermit-ignore-budget ./benchmarks/scripts/check-hermit-ignore-budget.sh
step pizza-el-golden ./benchmarks/scripts/compare-pizza-el-golden.sh
step tier-b-classification ./benchmarks/scripts/compare-classification-fixtures.sh
step hermit-tier-a cargo test -p ontologos-conformance --release --locked \
  --test hermit_generated \
  --test hermit_rdfs \
  --test hermit_rl \
  --test hermit_el \
  --test hermit_parser \
  --test hermit_wg_generated \
  --test dl_subsumption_cases \
  --test explain_benchmarks \
  -- \
  --test-threads=4 \
  --skip planned_engine_failure_scan \
  --skip ian_backjumping3_axiom_check_completes_within_budget
step conformance-inventory ./benchmarks/scripts/report-conformance-coverage.sh
step_optional dl-ofn-pass-rate ./benchmarks/scripts/report-dl-ofn-pass-rate.sh
step release-gates ./benchmarks/scripts/check-1.0-release-gates.sh
step hermit-parity-phases ./benchmarks/scripts/check-hermit-parity-phases.sh
step hermit-catalog ./benchmarks/scripts/check-hermit-catalog.sh
step phase9-closure cargo test -p ontologos-conformance --release --locked \
  --test phase9_closure -- --test-threads=1
step tier-c-gate ./benchmarks/scripts/compare-tier-c-gate.sh
step tier-c-harness ./benchmarks/scripts/compare-hermit-tier-c.sh
step reasonable ./benchmarks/scripts/compare-reasonable.sh
step el-incremental cargo test -p ontologos-el --test incremental_correctness --locked
step phase3-closure cargo test -p ontologos-conformance --test phase3_closure --locked
step phase6-closure cargo test -p ontologos-conformance --test phase6_closure --release --locked
step phase7-closure cargo test -p ontologos-conformance --test phase7_closure --release --locked
step build-cli cargo build -p ontologos-cli --release

# --- job: python (macOS / Linux) ---
step python-package bash -c '
  set -euo pipefail
  cd crates/ontologos-py
  python3 -m venv .venv
  # shellcheck disable=SC1091
  source .venv/bin/activate
  pip install -q '"'"'maturin>=1.7,<2.0'"'"' pytest '"'"'.[pandas,polars]'"'"'
  maturin develop --release
  pytest tests/ -q
'

# --- job: docs ---
step docs-build bash -c '
  set -euo pipefail
  pip3 install -q -r docs/requirements.txt
  ./docs/build-site.sh
'

echo "" | tee -a "$LOG"
echo "All local CI checks passed — $(date -u +%Y-%m-%dT%H:%M:%SZ)" | tee -a "$LOG"
