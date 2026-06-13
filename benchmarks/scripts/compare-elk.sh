#!/usr/bin/env bash
# Compare OntoLogos EL taxonomy against a committed golden file.
# Primary reference: regenerate golden from the in-house EL engine when updating baselines.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PIZZA="${ROOT}/benchmarks/data/pizza.owl"
GOLDEN="${ROOT}/benchmarks/data/pizza-el-golden.json"

if [[ ! -f "${PIZZA}" ]]; then
  echo "missing pizza.owl; run benchmarks/scripts/download.sh"
  exit 1
fi

cargo run --quiet -p ontologos-cli --release -- --profile el --format json classify "${PIZZA}" > /tmp/ontologos-pizza-el.json

if [[ -f "${GOLDEN}" ]]; then
  if [[ "${UPDATE_GOLDEN:-0}" == "1" ]]; then
    cp /tmp/ontologos-pizza-el.json "${GOLDEN}"
    echo "updated ${GOLDEN}"
  else
  python3 - <<'PY'
import json, sys
from pathlib import Path

golden = json.loads(Path("benchmarks/data/pizza-el-golden.json").read_text())
actual = json.loads(Path("/tmp/ontologos-pizza-el.json").read_text())

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
    cp /tmp/ontologos-pizza-el.json "${GOLDEN}"
    echo "wrote ${GOLDEN}"
  fi
fi
