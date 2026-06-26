#!/usr/bin/env bash
# Run HermiT parity scan tools (fast by default; --full for complete rescan).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT}"

# shellcheck source=burndown-process-guard.sh
source "${ROOT}/benchmarks/scripts/burndown-process-guard.sh"
burndown_guard_begin

BIN="$("${ROOT}/benchmarks/scripts/build-conformance-tools.sh")"

FULL=0
for arg in "$@"; do
  [[ "${arg}" == "--full" ]] && FULL=1
done

echo "==> Parity dashboard"
"${BIN}/parity_status"
echo

if [[ "${FULL}" -eq 1 ]]; then
  echo "==> Planned backlog (full engine audit)"
  "${BIN}/audit_planned_backlog" 2>/dev/null | head -20 || true
  echo
  echo "==> WG failures (all active)"
  "${BIN}/wg_failures" --all | head -30
  echo
  echo "==> Planned engine failures"
  "${BIN}/engine_failures" | head -20
  echo
  echo "==> Full axiom promotion scan"
  "${BIN}/promote_catalog"
else
  echo "==> Planned backlog (fast — metadata only)"
  "${BIN}/audit_planned_backlog" --fast 2>/dev/null | head -20 || true
  echo
  echo "==> WG failures (unpromoted only — use --full for all)"
  "${BIN}/wg_failures" | head -30
  echo
  echo "==> Tip: bash benchmarks/scripts/hermit-burndown.sh promote  # incremental"
fi
