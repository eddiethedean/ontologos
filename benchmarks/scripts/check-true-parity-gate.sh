#!/usr/bin/env bash
# Composite true-parity gate (Phase 8 final / post–Phase 9 burndown).
#
# true_parity_pct = min(
#   literal catalog green,   # runnable Java + WG catalog statuses / 1019
#   taxonomy_strict_pct,     # Tier C HermiT --max-extra 0
#   perf_gate_pct,           # ROADMAP DL perf targets
#   internal_port_pct,       # tableau.* / graph.* → alc unit tests
#   rules_test_pct           # RulesTest swrl active / catalog
# )
#
# Default: fail when true_parity_pct < ONTOLOGOS_TRUE_PARITY_MIN (100).
#
# Staged rollout (CI initial):
#   ONTOLOGOS_TRUE_PARITY_GATE=informational   — print WARN, exit 0
#   ONTOLOGOS_TRUE_PARITY_MIN=19               — floor at current baseline (~19%)
#
# Path to blocking CI:
#   1. informational @ floor matching current true_parity_pct (now)
#   2. raise ONTOLOGOS_TRUE_PARITY_MIN as sub-metrics improve (50 → 80 → 100)
#   3. ONTOLOGOS_TRUE_PARITY_GATE=blocking @ 100% for full HermiT replacement claim
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BIN="$("${ROOT}/benchmarks/scripts/build-conformance-tools.sh")"

METRICS="$("${BIN}/parity_status" --json)"
export METRICS_JSON="${METRICS}"

TRUE_PCT="$(python3 -c "import json, os; m=json.loads(os.environ['METRICS_JSON']); print(f\"{m['true_parity_pct']:.1f}\")")"
MIN_PCT="${ONTOLOGOS_TRUE_PARITY_MIN:-100}"
GATE_MODE="${ONTOLOGOS_TRUE_PARITY_GATE:-blocking}"

echo "True parity gate (Phase 8 final)"
echo "  true_parity_pct: ${TRUE_PCT}%"
echo "  minimum:         ${MIN_PCT}%"
echo "  mode:            ${GATE_MODE}"
python3 <<'PY'
import json, os
m = json.loads(os.environ["METRICS_JSON"])
rows = [
    ("literal catalog green", m["literal_green_pct"]),
    ("taxonomy strict (Tier C)", m["taxonomy_strict_pct"]),
    ("perf gate (Tier D)", m["perf_gate_pct"]),
    ("internal ports (B3)", m["internal_port_pct"]),
    ("rules test (SWRL)", m["rules_test_pct"]),
]
for label, pct in rows:
    print(f"  {label}: {pct:.1f}%")
print(f"  activatable #[ignore]: {m['activatable_ignored']}")
PY

if python3 <<'PY'
import json, os, sys
m = json.loads(os.environ["METRICS_JSON"])
true_pct = m["true_parity_pct"]
min_pct = float(os.environ.get("ONTOLOGOS_TRUE_PARITY_MIN", "100"))
sys.exit(0 if true_pct + 1e-9 >= min_pct else 1)
PY
then
  echo "OK  true parity ${TRUE_PCT}% >= ${MIN_PCT}%"
  exit 0
fi

MSG="true parity ${TRUE_PCT}% < ${MIN_PCT}% (bottleneck: see sub-metrics above)"
if [[ "${GATE_MODE}" == "informational" ]]; then
  echo "WARN ${MSG}" >&2
  echo "      informational mode — not blocking CI (raise ONTOLOGOS_TRUE_PARITY_MIN or set GATE=blocking when ready)" >&2
  exit 0
fi

echo "FAIL ${MSG}" >&2
echo "" >&2
echo "See docs/internal/parity-roadmap.md and:" >&2
echo "  bash benchmarks/scripts/hermit-burndown.sh status" >&2
exit 1
