#!/usr/bin/env bash
# Tier C strict HermiT cross-check (--max-extra 0) for HermiT cross-check corpora.
# Writes benchmarks/data/tier-c-strict-status.json with tier_c_strict_pct.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DATA="${ROOT}/benchmarks/data"
STATUS="${DATA}/tier-c-strict-status.json"
GOLDEN="${DATA}/dl-taxonomy-golden.json"
HERMIT="${HERMIT_JAR:-}"
REQUIRE="${ONTOLOGOS_REQUIRE_HERMIT_JAR:-0}"

write_status() {
  python3 - "${STATUS}" "${GOLDEN}" "$@" <<'PY'
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

status_path = Path(sys.argv[1])
golden_path = Path(sys.argv[2])
results = {}
for arg in sys.argv[3:]:
    corpus, passed = arg.split("\t", 1)
    results[corpus] = {"strict_pass": passed == "1"}

doc = json.load(open(golden_path))
run = list(results.keys())
if not run:
    pct = 0.0
else:
    passed = sum(1 for name in run if results[name].get("strict_pass"))
    pct = 100.0 * passed / len(run)

out = {
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "corpora": results,
    "tier_c_strict_pct": pct,
}
status_path.parent.mkdir(parents=True, exist_ok=True)
status_path.write_text(json.dumps(out, indent=2) + "\n")
print(f"tier_c_strict_pct={pct}")
PY
}

if [[ -z "${HERMIT}" ]] || [[ ! -f "${HERMIT}" ]]; then
  if [[ "${REQUIRE}" == "1" ]]; then
    echo "HermiT strict Tier C gate required: set HERMIT_JAR" >&2
    exit 1
  fi
  echo "skip strict Tier C cross-check: HERMIT_JAR not set"
  exit 0
fi

export ONTOLOGOS_STRICT_TAXONOMY=1
export RUN_SLOW_DL_GATES="${RUN_SLOW_DL_GATES:-0}"

CLI="${ROOT}/target/release/ontologos"
if [[ ! -x "${CLI}" ]]; then
  cargo build -q -p ontologos-cli --release
fi

TMP_ONTO="$(mktemp "${TMPDIR:-/tmp}/ontologos-strict.XXXXXX.json")"
TMP_HERMIT="$(mktemp "${TMPDIR:-/tmp}/hermit-strict.XXXXXX.ofn")"
TMP_HERMIT_JSON="$(mktemp "${TMPDIR:-/tmp}/hermit-strict.XXXXXX.json")"
trap 'rm -f "${TMP_ONTO}" "${TMP_HERMIT}" "${TMP_HERMIT_JSON}"' EXIT

STATUS_LINES=()
OVERALL=0

while read -r corpus profile optional; do
  if [[ "${optional}" == "True" ]] && [[ "${RUN_SLOW_DL_GATES:-0}" != "1" ]]; then
    echo "skip optional strict corpus ${corpus} (set RUN_SLOW_DL_GATES=1)"
    continue
  fi
  OWL="${DATA}/${corpus}"
  if [[ ! -f "${OWL}" ]]; then
    echo "missing corpus ${OWL}" >&2
    STATUS_LINES+=("${corpus}	0")
    OVERALL=1
    continue
  fi

  PREFIX="$(python3 -c "import json; print(json.load(open('${GOLDEN}'))['corpora']['${corpus}'].get('namespace_prefix',''))")"
  OWL_ABS="$(cd "$(dirname "${OWL}")" && pwd)/$(basename "${OWL}")"

  echo "Tier C strict: ${corpus} (profile=${profile})"
  "${CLI}" --profile "${profile}" --format json classify "${OWL}" >"${TMP_ONTO}"
  java -jar "${HERMIT}" -c -o "${TMP_HERMIT}" "file://${OWL_ABS}" >/dev/null 2>&1
  python3 "${ROOT}/benchmarks/scripts/hermit-taxonomy-to-json.py" "${TMP_HERMIT}" -o "${TMP_HERMIT_JSON}"

  NS_ARGS=()
  if [[ -n "${PREFIX}" ]]; then
    NS_ARGS=(--namespace-prefix "${PREFIX}")
  fi
  if python3 "${ROOT}/benchmarks/scripts/compare-taxonomy.py" \
    "${TMP_HERMIT_JSON}" "${TMP_ONTO}" \
    --max-missing 0 --max-extra 0 \
    "${NS_ARGS[@]}"; then
    STATUS_LINES+=("${corpus}	1")
  else
    STATUS_LINES+=("${corpus}	0")
    OVERALL=1
  fi
done < <(
  python3 - "${GOLDEN}" <<'PY'
import json
import sys

doc = json.load(open(sys.argv[1]))
for name, meta in doc["corpora"].items():
    if not meta.get("hermit_crosscheck", False):
        continue
    print(name, meta.get("profile", "dl"), meta.get("optional", False))
PY
)

write_status "${STATUS_LINES[@]}"

if [[ "${OVERALL}" -eq 0 ]]; then
  echo "Tier C strict gate: passed"
else
  echo "Tier C strict gate: failed (OntoLogos extras vs HermiT)" >&2
  exit 1
fi
