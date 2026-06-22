#!/usr/bin/env bash
# Tier C DL taxonomy gate — compare OntoLogos DL output to vendored golden baselines.
# Regenerate: UPDATE_GOLDEN=1 ./benchmarks/scripts/compare-dl-taxonomy.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DATA="${ROOT}/benchmarks/data"
GOLDEN="${DATA}/dl-taxonomy-golden.json"
CLI="${ROOT}/target/release/ontologos"

if [[ ! -f "${GOLDEN}" ]]; then
  echo "missing golden file: ${GOLDEN}" >&2
  exit 1
fi

if [[ ! -x "${CLI}" ]]; then
  echo "building release CLI for DL taxonomy gate"
  cargo build -q -p ontologos-cli --release
fi

TMP_JSON="$(mktemp "${TMPDIR:-/tmp}/ontologos-dl-tax.XXXXXX.json")"
trap 'rm -f "${TMP_JSON}"' EXIT

CORPORA="$(python3 -c "import json; print(' '.join(json.load(open('${GOLDEN}'))['corpora']))")"
for corpus in ${CORPORA}; do
  OWL="${DATA}/${corpus}"
  if [[ ! -f "${OWL}" ]]; then
    echo "missing corpus ${OWL}" >&2
    exit 1
  fi
  PROFILE="$(python3 -c "import json; print(json.load(open('${GOLDEN}'))['corpora']['${corpus}'].get('profile','dl'))")"
  echo "DL taxonomy gate: ${corpus} (profile=${PROFILE})"
  "${CLI}" --profile "${PROFILE}" --format json classify "${OWL}" >"${TMP_JSON}"

  if [[ "${UPDATE_GOLDEN:-0}" == "1" ]]; then
    python3 - <<PY
import json
from pathlib import Path
golden_path = Path("${GOLDEN}")
actual = json.loads(Path("${TMP_JSON}").read_text())
doc = json.loads(golden_path.read_text())
doc["corpora"]["${corpus}"] = {
    "profile": "${PROFILE}",
    "status": actual.get("status", "classified"),
    "subsumption_count": actual.get("subsumption_count", len(actual.get("subsumptions", []))),
    "subsumptions": actual["subsumptions"],
}
golden_path.write_text(json.dumps(doc, indent=2) + "\\n")
print("updated ${corpus} in", golden_path)
PY
  else
    python3 "${ROOT}/benchmarks/scripts/compare-taxonomy.py" \
      "${GOLDEN}" "${TMP_JSON}" --corpus-key "${corpus}"
  fi
done

echo "DL taxonomy gate: all corpora passed"
