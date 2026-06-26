#!/usr/bin/env bash
# HermiT parity phase gate — blocks v1.0.0 tag when in-scope catalog parity < 100%.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BIN="$("${ROOT}/benchmarks/scripts/build-conformance-tools.sh")"

METRICS="$("${BIN}/parity_status" --json)"
JAVA_PLANNED="$(python3 -c "import json,sys; m=json.load(sys.stdin); print(m['java_planned'])" <<<"${METRICS}")"
WG_PLANNED="$(python3 -c "import json,sys; m=json.load(sys.stdin); print(m['wg_planned'])" <<<"${METRICS}")"
PARITY="$(python3 -c "import json,sys; m=json.load(sys.stdin); print(f\"{m['parity_pct']:.1f}\")" <<<"${METRICS}")"
BACKLOG="$(python3 -c "import json,sys; m=json.load(sys.stdin); print(m['backlog'])" <<<"${METRICS}")"
IN_SCOPE="$(python3 -c "import json,sys; m=json.load(sys.stdin); print(m['in_scope_total'])" <<<"${METRICS}")"

echo "HermiT parity phase gate"
echo "  in_scope_total:  ${IN_SCOPE}"
echo "  java planned:    ${JAVA_PLANNED}"
echo "  wg planned:      ${WG_PLANNED}"
echo "  parity_pct:      ${PARITY}%"

if [[ "${JAVA_PLANNED}" -eq 0 ]] && [[ "${WG_PLANNED}" -eq 0 ]]; then
  echo "OK  catalog parity 100% (zero planned)"
else
  echo "FAIL catalog parity < 100% — ${BACKLOG} planned cases remain" >&2
  echo "" >&2
  echo "See ROADMAP.md § HermiT parity phases and:" >&2
  echo "  bash benchmarks/scripts/hermit-burndown.sh triage" >&2
  exit 1
fi

echo "All HermiT parity phase gates satisfied."
