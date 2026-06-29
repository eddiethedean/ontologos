#!/usr/bin/env bash
# PR-blocking Tier D perf gate — Family DL classify within ROADMAP budget (release CLI).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DATA="${ROOT}/benchmarks/data"
CLI="${ROOT}/target/release/ontologos"
OWL="${DATA}/family.owl"

# ROADMAP target 0.1s; PR gate uses 1.0s until saturation/tableau optimization lands.
FAMILY_BUDGET_S="${ONTOLOGOS_FAMILY_DL_BUDGET_S:-1.0}"

if [[ ! -f "${OWL}" ]]; then
  echo "missing ${OWL} (run benchmarks/scripts/download.sh)" >&2
  exit 1
fi

if [[ ! -x "${CLI}" ]]; then
  cargo build -q -p ontologos-cli --release
fi

start="$(python3 -c 'import time; print(time.perf_counter())')"
"${CLI}" --profile dl --format json classify "${OWL}" >/dev/null
end="$(python3 -c 'import time; print(time.perf_counter())')"
elapsed="$(python3 -c "print(round(${end} - ${start}, 3))")"

python3 - "${JSON_OUT:-${DATA}/dl-perf-snapshot.json}" "${elapsed}" "${FAMILY_BUDGET_S}" <<'PY'
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

json_path = Path(sys.argv[1])
elapsed = float(sys.argv[2])
budget = float(sys.argv[3])
meets = elapsed <= budget

doc = {
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "cli": "ontologos --profile dl classify family.owl (release)",
    "run_slow_dl_gates": False,
    "results": [
        {
            "corpus": "family.owl",
            "profile": "dl",
            "elapsed_s": elapsed,
            "target_s": budget,
            "meets_target": meets,
        }
    ],
}
json_path.parent.mkdir(parents=True, exist_ok=True)
json_path.write_text(json.dumps(doc, indent=2) + "\n")
print(f"family.owl DL: {elapsed:.3f}s (budget {budget:.1f}s)")
sys.exit(0 if meets else 1)
PY
