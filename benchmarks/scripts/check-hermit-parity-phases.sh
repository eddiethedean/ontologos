#!/usr/bin/env bash
# HermiT parity phase gate — blocks v1.0.0 tag when in-scope catalog parity < 100%.
# Phase 9: java_planned = 0 and wg_planned = 0 (see ROADMAP.md § HermiT parity phases).
# CI runs this informationally until Phase 9; then remove || true and make blocking.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CASES="${ROOT}/benchmarks/data/hermit/catalog/cases.json"
WG="${ROOT}/benchmarks/data/hermit/catalog/wg_cases.json"
FAIL=0

read_counts() {
  python3 - "$CASES" "$WG" <<'PY'
import json, sys
cases = json.load(open(sys.argv[1]))
wg = json.load(open(sys.argv[2]))
java_planned = sum(1 for c in cases if c["status"] == "planned")
wg_planned = sum(1 for c in wg if c["status"] == "planned")
internal = sum(1 for c in cases if c["status"] == "internal")
excluded = sum(1 for c in cases if c["status"] == "excluded")
migrated = sum(1 for c in cases if c["status"] == "migrated")
in_scope = (len(cases) - internal - excluded - migrated) + len(wg)
backlog = java_planned + wg_planned
parity = 100.0 * (1.0 - backlog / in_scope) if in_scope else 0.0
print(java_planned, wg_planned, int(in_scope), f"{parity:.1f}")
PY
}

IFS=' ' read -r JAVA_PLANNED WG_PLANNED IN_SCOPE PARITY_PCT < <(read_counts)

echo "HermiT parity phase gate"
echo "  in_scope_total:  ${IN_SCOPE}"
echo "  java planned:    ${JAVA_PLANNED}"
echo "  wg planned:      ${WG_PLANNED}"
echo "  parity_pct:      ${PARITY_PCT}%"

if [[ "${JAVA_PLANNED}" -eq 0 ]] && [[ "${WG_PLANNED}" -eq 0 ]]; then
  echo "OK  catalog parity 100% (zero planned)"
else
  echo "FAIL catalog parity < 100% — clear planned backlog (Phases 2–5)" >&2
  FAIL=1
fi

if [[ "${FAIL}" -ne 0 ]]; then
  echo "" >&2
  echo "See ROADMAP.md § HermiT parity phases and:" >&2
  echo "  bash benchmarks/scripts/audit-planned-backlog.sh" >&2
  exit 1
fi

echo "All HermiT parity phase gates satisfied."
