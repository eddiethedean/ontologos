#!/usr/bin/env bash
# Unified HermiT burndown workflow — status, triage, promote, test.
#
# Usage:
#   hermit-burndown.sh status              # fast parity dashboard (<1s)
#   hermit-burndown.sh triage              # unpromoted WG failures + fast backlog audit
#   hermit-burndown.sh triage --full       # full WG scan + engine audit (slow)
#   hermit-burndown.sh promote             # incremental promote (unpromoted only)
#   hermit-burndown.sh promote --full      # rescan entire catalog for promotion
#   hermit-burndown.sh resync              # rewrite promoted lists to passing-only set
#   hermit-burndown.sh test                # blocking CI conformance subset
#   hermit-burndown.sh test-full           # failure-first full suite
#   hermit-burndown.sh cleanup             # stop stale burndown/cargo processes
#   hermit-burndown.sh loop                # recommended fix-verify loop
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT}"

# shellcheck source=burndown-process-guard.sh
source "${ROOT}/benchmarks/scripts/burndown-process-guard.sh"

export ONTOLOGOS_DL_BUDGET_SECS="${ONTOLOGOS_DL_BUDGET_SECS:-30}"
export ONTOLOGOS_DL_MAX_WORKERS="${ONTOLOGOS_DL_MAX_WORKERS:-10}"
export ONTOLOGOS_SCAN_THREADS="${ONTOLOGOS_SCAN_THREADS:-10}"

CMD="${1:-status}"
shift || true

BIN="$("${ROOT}/benchmarks/scripts/build-conformance-tools.sh")"

case "${CMD}" in
  status)
    "${BIN}/parity_status" "$@"
    ;;

  cleanup)
    burndown_guard_cleanup_cmd
    ;;

  triage)
    burndown_guard_begin
    FULL=0
    for arg in "$@"; do
      [[ "${arg}" == "--full" ]] && FULL=1
    done
    "${BIN}/parity_status"
    echo ""
    if [[ "${FULL}" -eq 1 ]]; then
      echo "==> WG failures (all active)"
      "${BIN}/wg_failures" --all | head -40
      echo ""
      echo "==> Planned backlog (full engine audit)"
      "${BIN}/audit_planned_backlog" 2>/dev/null | head -30 || true
    else
      echo "==> WG failures (unpromoted only — use triage --full for all)"
      "${BIN}/wg_failures" | head -40
      echo ""
      echo "==> Planned backlog (fast metadata)"
      "${BIN}/audit_planned_backlog" --fast 2>/dev/null | head -20 || true
    fi
    ;;

  promote)
    burndown_guard_begin
    FULL=0
    for arg in "$@"; do
      [[ "${arg}" == "--full" ]] && FULL=1
    done
    INC_FLAG=(--incremental)
    [[ "${FULL}" -eq 1 ]] && INC_FLAG=()
    echo "==> Promote axiom cases"
    "${BIN}/promote_catalog" "${INC_FLAG[@]}"
    echo "==> Promote WG cases"
    "${BIN}/promote_wg" "${INC_FLAG[@]}"
    if [[ "${FULL}" -eq 0 ]]; then
      echo "==> Refresh catalog artifacts (promote-only)"
      python3 tests/hermit/generate_catalog.py --promote-only
      python3 tests/hermit/generate_catalog.py --promote-wg-only
    fi
    echo "==> Updated status"
    "${BIN}/parity_status"
    ;;

  resync)
    burndown_guard_begin
    export ONTOLOGOS_SCAN_THREADS="${ONTOLOGOS_SCAN_THREADS:-1}"
    echo "==> Sync promoted lists (passing-only @ ${ONTOLOGOS_DL_BUDGET_SECS}s)"
    "${BIN}/sync_promoted"
    echo "==> Refresh catalog artifacts (promote-only)"
    python3 tests/hermit/generate_catalog.py --promote-only
    python3 tests/hermit/generate_catalog.py --promote-wg-only
    echo "==> Updated status"
    "${BIN}/parity_status"
    ;;

  test)
    burndown_guard_begin
    export ONTOLOGOS_CI_PROMOTED_ONLY=1
    cargo test -p ontologos-conformance --release --locked \
      --test hermit_generated \
      --test hermit_rdfs \
      --test hermit_rl \
      --test hermit_el \
      --test hermit_parser \
      --test hermit_wg_generated \
      -- \
      --test-threads=4 \
      "$@"
    ;;

  test-full)
    burndown_guard_begin
    bash "${ROOT}/benchmarks/scripts/run-hermit-full-suite.sh" "$@"
    ;;

  loop)
    cat <<EOF
HermiT burndown — daily fix-verify loop
=======================================
Full guide: docs/guides/hermit-burndown.md
           (published: https://ontologos.readthedocs.io/en/latest/guides/hermit-burndown.html)

If a previous run was interrupted:
  bash benchmarks/scripts/hermit-burndown.sh cleanup

Why two modes?
  • test-full  = truth (every active case) — drives parity_pct toward 100%
  • test       = CI gate (promoted cases only) — keeps main green

Loop:
  1. bash benchmarks/scripts/hermit-burndown.sh status
  2. bash benchmarks/scripts/hermit-burndown.sh triage
  3. Fix engine / harvest assertions / hand-port (see guide § What kind of fix?)
  4. cargo test -p ontologos-conformance --release --test wg_phase4_check  # WG fixes
  5. bash benchmarks/scripts/hermit-burndown.sh promote   # record passes for CI
  6. bash benchmarks/scripts/hermit-burndown.sh test
  7. (optional) bash benchmarks/scripts/hermit-burndown.sh test-full

Environment:
  ONTOLOGOS_DL_BUDGET_SECS=30   # CI parity (default here)
  ONTOLOGOS_DL_BUDGET_SECS=120  # final promotion / nightly
EOF
    ;;

  -h|--help|help)
    echo "HermiT burndown workflow — see docs/guides/hermit-burndown.md"
    echo ""
    sed -n '4,13p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
    ;;

  *)
    echo "unknown command: ${CMD}" >&2
    echo "run: hermit-burndown.sh --help" >&2
    exit 1
    ;;
esac
