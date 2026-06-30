#!/usr/bin/env bash
# Diff HermiT vs OntoLogos taxonomy edges for Tier C HermiT cross-check corpora.
# Usage: HERMIT_JAR=benchmarks/data/hermit.jar ./benchmarks/scripts/audit-taxonomy-extras.sh [corpus ...]
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DATA="${ROOT}/benchmarks/data"
GOLDEN="${DATA}/dl-taxonomy-golden.json"
CLI="${ROOT}/target/release/ontologos"
HERMIT="${HERMIT_JAR:-}"

if [[ -z "${HERMIT}" ]] || [[ ! -f "${HERMIT}" ]]; then
  echo "set HERMIT_JAR to a HermiT.jar path" >&2
  exit 1
fi
if ! command -v java >/dev/null 2>&1; then
  echo "java not found" >&2
  exit 1
fi

if [[ ! -x "${CLI}" ]]; then
  cargo build -q -p ontologos-cli --release
fi

TMP_ONTO="$(mktemp "${TMPDIR:-/tmp}/ontologos-audit.XXXXXX.json")"
TMP_HERMIT="$(mktemp "${TMPDIR:-/tmp}/hermit-audit.XXXXXX.ofn")"
TMP_HERMIT_JSON="$(mktemp "${TMPDIR:-/tmp}/hermit-audit.XXXXXX.json")"
trap 'rm -f "${TMP_ONTO}" "${TMP_HERMIT}" "${TMP_HERMIT_JSON}"' EXIT

if [[ "$#" -gt 0 ]]; then
  CORPORA=("$@")
else
  readarray -t CORPORA < <(
    python3 - "${GOLDEN}" <<'PY'
import json
import sys

doc = json.load(open(sys.argv[1]))
for name, meta in doc["corpora"].items():
    if meta.get("hermit_crosscheck", False):
        print(name)
PY
  )
fi

OWL_THING='http://www.w3.org/2002/07/owl#Thing'

for corpus in "${CORPORA[@]}"; do
  OWL="${DATA}/${corpus}"
  if [[ ! -f "${OWL}" ]]; then
    echo "skip ${corpus}: missing ${OWL}" >&2
    continue
  fi

  PROFILE="$(python3 -c "import json; print(json.load(open('${GOLDEN}'))['corpora']['${corpus}'].get('profile','dl'))")"
  PREFIX="$(python3 -c "import json; print(json.load(open('${GOLDEN}'))['corpora']['${corpus}'].get('namespace_prefix',''))")"

  OWL_ABS="$(cd "$(dirname "${OWL}")" && pwd)/$(basename "${OWL}")"
  echo "=== ${corpus} (profile=${PROFILE}) ==="

  "${CLI}" --profile "${PROFILE}" --format json classify "${OWL}" >"${TMP_ONTO}"
  java -jar "${HERMIT}" -c -o "${TMP_HERMIT}" "file://${OWL_ABS}" >/dev/null 2>&1
  python3 "${ROOT}/benchmarks/scripts/hermit-taxonomy-to-json.py" "${TMP_HERMIT}" -o "${TMP_HERMIT_JSON}"

  python3 - "${TMP_HERMIT_JSON}" "${TMP_ONTO}" "${PREFIX}" <<'PY'
import json
import sys

hermit_path, onto_path, prefix = sys.argv[1:4]
OWL_THING = "http://www.w3.org/2002/07/owl#Thing"

def norm(iri: str) -> str:
    return iri.replace("%23", "#")

def pairs(doc, p):
    s = {(norm(a), norm(b)) for a, b in doc.get("subsumptions", [])}
    s = {(a, b) for a, b in s if b != OWL_THING}
    if p:
        s = {(a, b) for a, b in s if a.startswith(p)}
    return s

h = pairs(json.load(open(hermit_path)), prefix)
o = pairs(json.load(open(onto_path)), prefix)
extra = sorted(o - h)
missing = sorted(h - o)
print(f"HermiT edges: {len(h)}")
print(f"OntoLogos edges: {len(o)}")
print(f"Missing (HermiT not in OntoLogos): {len(missing)}")
print(f"Extra (OntoLogos not in HermiT): {len(extra)}")
if missing:
    print("Missing sample:")
    for a, b in missing[:10]:
        print(f"  {a.split('#')[-1]} -> {b.split('#')[-1]}")
if extra:
    print("Extra sample:")
    for a, b in extra[:20]:
        print(f"  {a.split('#')[-1]} -> {b.split('#')[-1]}")
    if len(extra) > 20:
        print(f"  ... and {len(extra) - 20} more")
PY
  echo
done
