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

# Phase 6 — Tier B classification corpora (informational sub-gate).
PHASE6_FAIL=0
TIER_B_SCRIPT="${ROOT}/benchmarks/scripts/compare-classification-fixtures.sh"
HERMIT_RES="${ROOT}/benchmarks/data/hermit/reasoner/res"
if [[ -x "${TIER_B_SCRIPT}" ]]; then
  echo "OK  Phase 6: compare-classification-fixtures.sh executable"
else
  echo "FAIL Phase 6: missing ${TIER_B_SCRIPT}" >&2
  PHASE6_FAIL=1
fi
for fixture in pizza.xml wine.xml galen-ians-full-undoctored.xml propreo.xml; do
  if [[ ! -f "${HERMIT_RES}/${fixture}" ]] || [[ ! -f "${HERMIT_RES}/${fixture}.txt" ]]; then
    echo "FAIL Phase 6: missing vendored fixture ${fixture}" >&2
    PHASE6_FAIL=1
  fi
done
HERMIT_EL_TESTS="$(grep -c '#\[test\]' "${ROOT}/crates/ontologos-conformance/tests/hermit_el.rs" 2>/dev/null || echo 0)"
if [[ "${HERMIT_EL_TESTS}" -ge 5 ]]; then
  echo "OK  Phase 6: hermit_el.rs has ${HERMIT_EL_TESTS} tests (≥ 5)"
else
  echo "FAIL Phase 6: hermit_el.rs has ${HERMIT_EL_TESTS} tests (need ≥ 5)" >&2
  PHASE6_FAIL=1
fi
if [[ "${PHASE6_FAIL}" -eq 0 ]]; then
  echo "OK  Phase 6 Tier B classification corpora"
else
  echo "WARN Phase 6 Tier B incomplete (see ROADMAP Phase 6)" >&2
fi

# Phase 7 — Tier C external proof (informational sub-gate).
PHASE7_FAIL=0
for script in compare-tier-c-gate.sh compare-dl-hermit-crosscheck.sh download-hermit-jar.sh benchmark-dl-perf.sh; do
  path="${ROOT}/benchmarks/scripts/${script}"
  if [[ -x "${path}" ]]; then
    echo "OK  Phase 7: ${script} executable"
  else
    echo "FAIL Phase 7: missing ${path}" >&2
    PHASE7_FAIL=1
  fi
done
if grep -q 'ONTOLOGOS_REQUIRE_HERMIT_JAR' "${ROOT}/benchmarks/scripts/compare-dl-hermit-crosscheck.sh" 2>/dev/null; then
  echo "OK  Phase 7: compare-dl-hermit-crosscheck.sh supports ONTOLOGOS_REQUIRE_HERMIT_JAR"
else
  echo "FAIL Phase 7: compare-dl-hermit-crosscheck.sh missing REQUIRE mode" >&2
  PHASE7_FAIL=1
fi
if [[ -f "${ROOT}/crates/ontologos-conformance/tests/phase7_closure.rs" ]]; then
  echo "OK  Phase 7: phase7_closure.rs present"
else
  echo "FAIL Phase 7: missing phase7_closure.rs" >&2
  PHASE7_FAIL=1
fi
if [[ "${PHASE7_FAIL}" -eq 0 ]]; then
  echo "OK  Phase 7 Tier C external proof harness"
else
  echo "WARN Phase 7 Tier C incomplete (see ROADMAP Phase 7)" >&2
fi
echo ""

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
