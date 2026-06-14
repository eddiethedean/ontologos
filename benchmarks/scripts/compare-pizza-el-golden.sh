#!/usr/bin/env bash
# Pizza EL golden regression gate — NOT an ELK or whelk diff.
# Classifies pizza.owl with in-house EL and compares to committed golden JSON.
# Regenerate golden: UPDATE_GOLDEN=1 ./benchmarks/scripts/compare-pizza-el-golden.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PIZZA="${ROOT}/benchmarks/data/pizza.owl"
GOLDEN="${ROOT}/benchmarks/data/pizza-el-golden.json"

if [[ ! -f "${PIZZA}" ]]; then
  echo "missing pizza.owl; run benchmarks/scripts/download.sh"
  exit 1
fi

TMP_JSON="$(mktemp "${TMPDIR:-/tmp}/ontologos-pizza-el.XXXXXX.json")"
trap 'rm -f "${TMP_JSON}"' EXIT

cargo run --quiet -p ontologos-cli --release -- --profile el --format json classify "${PIZZA}" > "${TMP_JSON}"

if [[ -f "${GOLDEN}" ]]; then
  if [[ "${UPDATE_GOLDEN:-0}" == "1" ]]; then
    cp "${TMP_JSON}" "${GOLDEN}"
    echo "updated ${GOLDEN}"
  else
  python3 - <<PY
import json, sys
from pathlib import Path

golden = json.loads(Path("benchmarks/data/pizza-el-golden.json").read_text())
actual = json.loads(Path("${TMP_JSON}").read_text())

g = set(map(tuple, golden["subsumptions"]))
a = set(map(tuple, actual["subsumptions"]))
missing = g - a
extra = a - g
if missing or extra:
    print(f"mismatch: missing={len(missing)} extra={len(extra)}", file=sys.stderr)
    sys.exit(1)
print(f"ok: {len(a)} subsumptions match golden")
PY
  fi
else
  echo "no golden file at ${GOLDEN}; run with UPDATE_GOLDEN=1 to create"
  if [[ "${UPDATE_GOLDEN:-0}" == "1" ]]; then
    cp "${TMP_JSON}" "${GOLDEN}"
    echo "wrote ${GOLDEN}"
  fi
fi
