#!/usr/bin/env bash
# Optional Tier C cross-check: compare OntoLogos DL taxonomy to HermiT JAR output.
# Requires HERMIT_JAR and java. Not run in default PR CI.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DATA="${ROOT}/benchmarks/data"
GOLDEN="${DATA}/dl-taxonomy-golden.json"
CLI="${ROOT}/target/release/ontologos"
HERMIT="${HERMIT_JAR:-}"

if [[ -z "${HERMIT}" ]] || [[ ! -f "${HERMIT}" ]]; then
  echo "skip HermiT cross-check: set HERMIT_JAR to a HermiT.jar path"
  exit 0
fi
if ! command -v java >/dev/null 2>&1; then
  echo "skip HermiT cross-check: java not found"
  exit 0
fi

if [[ ! -x "${CLI}" ]]; then
  cargo build -q -p ontologos-cli --release
fi

TMP_ONTO="$(mktemp "${TMPDIR:-/tmp}/ontologos-dl.XXXXXX.json")"
TMP_HERMIT="$(mktemp "${TMPDIR:-/tmp}/hermit-tax.XXXXXX.ofn")"
TMP_HERMIT_JSON="$(mktemp "${TMPDIR:-/tmp}/hermit-tax.XXXXXX.json")"
trap 'rm -f "${TMP_ONTO}" "${TMP_HERMIT}" "${TMP_HERMIT_JSON}"' EXIT

python3 - "${GOLDEN}" <<'PY' | while read -r corpus profile optional crosscheck; do
import json, sys
doc = json.load(open(sys.argv[1]))
for name, meta in doc["corpora"].items():
    if not meta.get("hermit_crosscheck", False):
        continue
    print(name, meta.get("profile", "dl"), meta.get("optional", False), "1")
PY
  if [[ "${optional}" == "True" ]] && [[ "${RUN_SLOW_DL_GATES:-0}" != "1" ]]; then
    echo "skip optional corpus ${corpus} (set RUN_SLOW_DL_GATES=1)"
    continue
  fi
  OWL="${DATA}/${corpus}"
  if [[ ! -f "${OWL}" ]]; then
    echo "skip ${corpus}: missing file" >&2
    continue
  fi
  echo "HermiT cross-check: ${corpus} (profile=${profile})"
  "${CLI}" --profile "${profile}" --format json classify "${OWL}" >"${TMP_ONTO}"
  java -jar "${HERMIT}" -c -o "${TMP_HERMIT}" "file://${OWL}" 2>/dev/null
  python3 "${ROOT}/benchmarks/scripts/hermit-taxonomy-to-json.py" "${TMP_HERMIT}" -o "${TMP_HERMIT_JSON}"
  PREFIX="$(python3 -c "import json; print(json.load(open('${GOLDEN}'))['corpora'].get('${corpus}', {}).get('namespace_prefix', ''))")"
  NS_ARGS=()
  if [[ -n "${PREFIX}" ]]; then
    NS_ARGS=(--namespace-prefix "${PREFIX}")
  fi
  python3 "${ROOT}/benchmarks/scripts/compare-taxonomy.py" \
    "${TMP_HERMIT_JSON}" "${TMP_ONTO}" \
    --max-missing 0 --max-extra 100 \
    "${NS_ARGS[@]}"
done

echo "HermiT cross-check: done"
