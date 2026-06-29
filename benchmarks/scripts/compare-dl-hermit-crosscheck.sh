#!/usr/bin/env bash
# Optional Tier C cross-check: HermiT JAR ⊆ OntoLogos DL taxonomy.
# Requires HERMIT_JAR and java. Nightly: ONTOLOGOS_REQUIRE_HERMIT_JAR=1.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DATA="${ROOT}/benchmarks/data"
GOLDEN="${DATA}/dl-taxonomy-golden.json"
CLI="${ROOT}/target/release/ontologos"
HERMIT="${HERMIT_JAR:-}"
REQUIRE="${ONTOLOGOS_REQUIRE_HERMIT_JAR:-0}"

require_fail() {
  if [[ "${REQUIRE}" == "1" ]]; then
    echo "HermiT cross-check required: $*" >&2
    exit 1
  fi
  echo "skip HermiT cross-check: $*"
  exit 0
}

if [[ -z "${HERMIT}" ]] || [[ ! -f "${HERMIT}" ]]; then
  require_fail "set HERMIT_JAR to a HermiT.jar path (see benchmarks/scripts/download-hermit-jar.sh)"
fi
if ! command -v java >/dev/null 2>&1; then
  require_fail "java not found"
fi

if [[ ! -x "${CLI}" ]]; then
  cargo build -q -p ontologos-cli --release
fi

TMP_ONTO="$(mktemp "${TMPDIR:-/tmp}/ontologos-dl.XXXXXX.json")"
TMP_HERMIT="$(mktemp "${TMPDIR:-/tmp}/hermit-tax.XXXXXX.ofn")"
TMP_HERMIT_JSON="$(mktemp "${TMPDIR:-/tmp}/hermit-tax.XXXXXX.json")"
trap 'rm -f "${TMP_ONTO}" "${TMP_HERMIT}" "${TMP_HERMIT_JSON}"' EXIT

while read -r corpus profile optional crosscheck; do
  if [[ "${optional}" == "True" ]] && [[ "${RUN_SLOW_DL_GATES:-0}" != "1" ]]; then
    echo "skip optional corpus ${corpus} (set RUN_SLOW_DL_GATES=1)"
    continue
  fi
  OWL="${DATA}/${corpus}"
  if [[ ! -f "${OWL}" ]]; then
    if [[ "${optional}" == "True" ]]; then
      echo "skip optional ${corpus}: missing file" >&2
      continue
    fi
    echo "missing required corpus ${OWL}" >&2
    exit 1
  fi

  OWL_ABS="$(cd "$(dirname "${OWL}")" && pwd)/$(basename "${OWL}")"
  FILE_URI="file://${OWL_ABS}"

  echo "HermiT cross-check: ${corpus} (profile=${profile})"
  "${CLI}" --profile "${profile}" --format json classify "${OWL}" >"${TMP_ONTO}"
  java -jar "${HERMIT}" -c -o "${TMP_HERMIT}" "${FILE_URI}"
  python3 "${ROOT}/benchmarks/scripts/hermit-taxonomy-to-json.py" "${TMP_HERMIT}" -o "${TMP_HERMIT_JSON}"

  read -r PREFIX MAX_EXTRA < <(
    python3 - "${GOLDEN}" "${corpus}" "${TMP_HERMIT_JSON}" "${TMP_ONTO}" <<'PY'
import json
import math
import os
import sys

golden_path, corpus, hermit_path, onto_path = sys.argv[1:5]
strict = os.environ.get("ONTOLOGOS_STRICT_TAXONOMY", "0") == "1"
doc = json.load(open(golden_path))
prefix = doc["corpora"].get(corpus, {}).get("namespace_prefix", "")
hermit = json.load(open(hermit_path))
onto = json.load(open(onto_path))

def filter_ns(pairs, p):
    if not p:
        return pairs
    return [(a, b) for a, b in pairs if a.startswith(p)]

hermit_pairs = filter_ns(hermit.get("subsumptions", []), prefix)
onto_pairs = filter_ns(onto.get("subsumptions", []), prefix)
hermit_count = len(hermit_pairs)
onto_count = len(onto_pairs)
if strict:
    max_extra = 0
else:
    max_extra = max(5, math.ceil(hermit_count * 0.01))
    if onto_count > hermit_count:
        max_extra = max(max_extra, onto_count - hermit_count)
print(prefix, max_extra)
PY
  )

  NS_ARGS=()
  if [[ -n "${PREFIX}" ]]; then
    NS_ARGS=(--namespace-prefix "${PREFIX}")
  fi
  python3 "${ROOT}/benchmarks/scripts/compare-taxonomy.py" \
    "${TMP_HERMIT_JSON}" "${TMP_ONTO}" \
    --max-missing 0 --max-extra "${MAX_EXTRA}" \
    "${NS_ARGS[@]}"
done < <(
  python3 - "${GOLDEN}" <<'PY'
import json
import sys

doc = json.load(open(sys.argv[1]))
for name, meta in doc["corpora"].items():
    if not meta.get("hermit_crosscheck", False):
        continue
    print(
        name,
        meta.get("profile", "dl"),
        meta.get("optional", False),
        "1",
    )
PY
)

echo "HermiT cross-check: done"
