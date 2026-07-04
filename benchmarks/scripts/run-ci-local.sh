#!/usr/bin/env bash
# Mirror .github/workflows/ci.yml locally (skips windows-only jobs).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

export CARGO_TERM_COLOR=always
export CARGO_INCREMENTAL=0
export ONTOLOGOS_DL_BUDGET_SECS=30
export ONTOLOGOS_WG_SHORTCUTS=1
export ONTOLOGOS_CONFORMANCE=1
export ONTOLOGOS_REPO_ROOT="$ROOT"

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
step test-workspace cargo test --workspace --exclude ontologos-conformance --exclude ontologos-contract --locked
step contract cargo test -p ontologos-contract --release --locked
step native-bindings cargo build -p ontologos-jni -p ontologos-dotnet -p ontologos-c --release --locked
step hermit-ignore-budget ./benchmarks/scripts/check-hermit-ignore-budget.sh
step pizza-el-golden ./benchmarks/scripts/compare-pizza-el-golden.sh
step tier-b-classification ./benchmarks/scripts/compare-classification-fixtures.sh
step conformance-inventory ./benchmarks/scripts/report-conformance-coverage.sh
step_optional dl-ofn-pass-rate ./benchmarks/scripts/report-dl-ofn-pass-rate.sh
step pr-gates ./benchmarks/scripts/check-pr-gates.sh
step hermit-parity-phases ./benchmarks/scripts/check-hermit-parity-phases.sh
step hermit-catalog ./benchmarks/scripts/check-hermit-catalog.sh
step tier-c-gate ./benchmarks/scripts/compare-tier-c-gate.sh
step tier-c-harness ./benchmarks/scripts/compare-hermit-tier-c.sh
step reasonable ./benchmarks/scripts/compare-reasonable.sh
step el-incremental cargo test -p ontologos-el --test incremental_correctness --locked
step build-cli cargo build -p ontologos-cli --release
step_optional tier-c-strict bash -c './benchmarks/scripts/compare-tier-c-strict-family.sh'
step tier-d-perf ./benchmarks/scripts/compare-tier-d-perf-gate.sh

# --- job: msrv ---
if rustup run 1.88 cargo --version >/dev/null 2>&1; then
  step msrv-test rustup run 1.88 cargo test --workspace --exclude ontologos-conformance --exclude ontologos-contract --locked
  step msrv-contract rustup run 1.88 cargo test -p ontologos-contract --release --locked
  step msrv-native rustup run 1.88 cargo build -p ontologos-jni -p ontologos-dotnet -p ontologos-c --release --locked
  step msrv-pizza ./benchmarks/scripts/compare-pizza-el-golden.sh
else
  echo "SKIP: rustup toolchain 1.88 not installed (rustup toolchain install 1.88)" | tee -a "$LOG"
fi

# --- job: node ---
if command -v node >/dev/null 2>&1 && command -v npm >/dev/null 2>&1; then
  if command -v wasm-pack >/dev/null 2>&1; then
    step node-wasm bash "$ROOT/scripts/ci-node.sh"
  else
    step node bash -c '
      set -euo pipefail
      cd crates/ontologos-node
      npm install
      npm run build
      npm test
    '
    echo "SKIP: wasm-pack not installed (install for full node job)" | tee -a "$LOG"
  fi
else
  echo "SKIP: node/npm not installed" | tee -a "$LOG"
fi

# --- job: bindings ---
step bindings bash "$ROOT/scripts/ci-bindings.sh"

# --- job: python ---
step python-package bash -c '
  set -euo pipefail
  cd crates/ontologos-py
  python3 -m venv .venv
  # shellcheck disable=SC1091
  source .venv/bin/activate
  pip install -q '"'"'maturin>=1.7,<2.0'"'"' pytest '"'"'.[pandas,polars]'"'"'
  maturin develop --release
  pytest tests/ -q
  pip install -q pyright
  pyright python tests
'

# --- job: docs ---
step docs-build bash -c '
  set -euo pipefail
  pip3 install -q -r docs/requirements.txt
  ./docs/build-site.sh
'

# --- local-only deep conformance (not in default GitHub check job) ---
step_optional hermit-tier-a cargo test -p ontologos-conformance --release --locked \
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
step_optional phase9-closure cargo test -p ontologos-conformance --release --locked \
  --test phase9_closure -- --test-threads=1
step_optional phase3-closure cargo test -p ontologos-conformance --test phase3_closure --locked
step_optional phase6-closure cargo test -p ontologos-conformance --test phase6_closure --release --locked
step_optional phase7-closure cargo test -p ontologos-conformance --test phase7_closure --release --locked
step_optional true-parity-gate ./benchmarks/scripts/check-true-parity-gate.sh
step_optional release-gates ./benchmarks/scripts/check-1.0-release-gates.sh

echo "" | tee -a "$LOG"
echo "All local CI checks passed — $(date -u +%Y-%m-%dT%H:%M:%SZ)" | tee -a "$LOG"
