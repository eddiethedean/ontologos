#!/usr/bin/env bash
# Fail if family_dl Criterion bench regresses more than 10% vs committed baseline.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BASELINE="${ROOT}/benchmarks/data/criterion-family-dl-baseline.json"
PIZZA_BASELINE="${ROOT}/benchmarks/data/criterion-pizza-dl-baseline.json"
MAX_REGRESSION_PCT="${ONTOLOGOS_CRITERION_MAX_REGRESSION_PCT:-10}"

check_bench() {
  local label="$1"
  local baseline_path="$2"
  local pattern="$3"

  if [[ ! -f "${baseline_path}" ]]; then
    echo "skip ${label} criterion regression: no baseline at ${baseline_path}" >&2
    return 0
  fi

  cargo bench -p ontologos-dl --bench classify -- "${pattern}" --noplot 2>&1 | tee "${OUT}" >/dev/null

  local measured_ns
  measured_ns="$(python3 - "${OUT}" "${label}" <<'PY'
import re
import sys
from pathlib import Path

text = Path(sys.argv[1]).read_text()
label = sys.argv[2]
m = re.search(rf"{label}\s+time:\s+\[[\d.]+\s*µs\s+([\d.]+)\s*µs", text)
if not m:
    m = re.search(rf"{label}\s+time:\s+\[[\d.]+\s*ms\s+([\d.]+)\s*ms", text)
    if m:
        print(int(float(m.group(1)) * 1_000_000))
        raise SystemExit(0)
    print("0", file=sys.stderr)
    raise SystemExit(1)
print(int(float(m.group(1)) * 1000))
PY
)"

  python3 - "${baseline_path}" "${measured_ns}" "${MAX_REGRESSION_PCT}" "${label}" <<'PY'
import json
import sys
from pathlib import Path

baseline_path = Path(sys.argv[1])
measured_ns = int(sys.argv[2])
max_pct = float(sys.argv[3])
label = sys.argv[4]
doc = json.loads(baseline_path.read_text())
key = f"{label}_ns" if f"{label}_ns" in doc else "family_dl_ns"
baseline_ns = int(doc[key])
limit = baseline_ns * (1.0 + max_pct / 100.0)
if measured_ns > limit:
    print(
        f"criterion regression: {label} {measured_ns}ns > {limit:.0f}ns "
        f"(baseline {baseline_ns}ns + {max_pct}%)",
        file=sys.stderr,
    )
    raise SystemExit(1)
print(f"criterion regression ok: {label} {measured_ns}ns (baseline {baseline_ns}ns)")
PY

  if [[ -n "${UPDATE_CRITERION_BASELINE:-}" ]]; then
    python3 - "${baseline_path}" "${measured_ns}" "${label}" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
measured_ns = int(sys.argv[2])
label = sys.argv[3]
key = f"{label}_ns"
doc = json.loads(path.read_text()) if path.exists() else {}
doc[key] = measured_ns
path.write_text(json.dumps(doc, indent=2) + "\n")
print(f"updated baseline: {key}={measured_ns}")
PY
  fi
}

if [[ ! -f "${BASELINE}" ]] && [[ ! -f "${PIZZA_BASELINE}" ]]; then
  echo "skip criterion regression: no baselines" >&2
  echo "  Run: UPDATE_CRITERION_BASELINE=1 $0" >&2
  exit 0
fi

cd "${ROOT}"
./benchmarks/scripts/download.sh >/dev/null 2>&1 || true

OUT="$(mktemp)"
trap 'rm -f "${OUT}"' EXIT

check_bench "family_dl" "${BASELINE}" "family_dl" || exit 1
check_bench "pizza_dl" "${PIZZA_BASELINE}" "pizza_dl" || exit 1
