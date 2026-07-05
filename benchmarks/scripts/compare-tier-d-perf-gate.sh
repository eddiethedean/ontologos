#!/usr/bin/env bash
# PR-blocking Tier D perf gate — Family DL classify within ROADMAP budget (release CLI).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DATA="${ROOT}/benchmarks/data"
CLI="${ROOT}/target/release/ontologos"
JSON_OUT="${JSON_OUT:-${DATA}/dl-perf-snapshot.json}"

# ROADMAP small-corpus target (Family < 100 ms); gate uses 0.1s with release CLI overhead.
FAMILY_BUDGET_S="${ONTOLOGOS_FAMILY_DL_BUDGET_S:-0.1}"
# Repeat timed runs and take the median to absorb subprocess / scheduler jitter.
PERF_ITERATIONS="${ONTOLOGOS_TIER_D_PERF_ITERATIONS:-3}"

if [[ ! -f "${DATA}/family.owl" ]]; then
  echo "missing ${DATA}/family.owl (run benchmarks/scripts/download.sh)" >&2
  exit 1
fi

if [[ ! -x "${CLI}" ]]; then
  cargo build -q -p ontologos-cli --release
fi

measure() {
  local corpus="$1"
  local profile="$2"
  local owl="${DATA}/${corpus}"
  if [[ ! -f "${owl}" ]]; then
    echo "skip ${corpus}: missing file" >&2
    return 0
  fi
  # Warm release CLI + OS caches before timed runs.
  "${CLI}" --profile "${profile}" --format json classify "${owl}" >/dev/null

  local samples=() elapsed
  local i
  for ((i = 0; i < PERF_ITERATIONS; i++)); do
    elapsed="$(python3 - "${CLI}" "${profile}" "${owl}" <<'PY'
import subprocess
import sys
import time

cli, profile, owl = sys.argv[1:4]
start = time.perf_counter()
subprocess.run(
    [cli, "--profile", profile, "--format", "json", "classify", owl],
    stdout=subprocess.DEVNULL,
    check=True,
)
print(round(time.perf_counter() - start, 3))
PY
)"
    samples+=("${elapsed}")
  done

  elapsed="$(python3 - "${samples[@]}" <<'PY'
import statistics
import sys

samples = [float(x) for x in sys.argv[1:] if x]
print(round(statistics.median(samples), 3))
PY
)"
  echo "${corpus}|${profile}|${elapsed}"
}

ROWS=()
while IFS= read -r row; do
  [[ -n "${row}" ]] && ROWS+=("${row}")
done < <(
  measure "family.owl" "dl"
  measure "pizza.owl" "dl"
  measure "go-subset.owl" "dl"
)

python3 - "${JSON_OUT}" "${FAMILY_BUDGET_S}" "${ROWS[@]}" <<'PY'
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

json_path = Path(sys.argv[1])
family_budget = float(sys.argv[2])
rows = [r.split("|", 2) for r in sys.argv[3:] if r]

targets = {
    "family.owl": {"target_s": family_budget, "gate": True},
    "pizza.owl": {"target_s": 30.0, "gate": False},
    "go-subset.owl": {"target_s": 10.0, "gate": False},
}

results = []
gate_ok = True
for corpus, profile, elapsed_s in rows:
    elapsed = float(elapsed_s)
    meta = targets.get(corpus, {"target_s": None, "gate": False})
    target = meta["target_s"]
    meets = (target is None) or (elapsed <= target)
    if meta.get("gate") and not meets:
        gate_ok = False
    results.append(
        {
            "corpus": corpus,
            "profile": profile,
            "elapsed_s": elapsed,
            "target_s": target,
            "meets_target": meets,
            "pr_gate": meta.get("gate", False),
        }
    )

doc = {
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "cli": "ontologos --profile dl classify (release)",
    "run_slow_dl_gates": False,
    "results": results,
}
json_path.parent.mkdir(parents=True, exist_ok=True)
json_path.write_text(json.dumps(doc, indent=2) + "\n")

for r in results:
    tag = "PR gate" if r.get("pr_gate") else "snapshot"
    status = "ok" if r["meets_target"] else "SLOW"
    target = "" if r["target_s"] is None else f"{r['target_s']:.1f}s"
    print(f"{r['corpus']} ({tag}): {r['elapsed_s']:.3f}s / {target} ({status})")

sys.exit(0 if gate_ok else 1)
PY
