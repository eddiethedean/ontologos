#!/usr/bin/env bash
# Measure DL classification wall time for Tier C corpora (release CLI).
# Informational in nightly CI; run locally to refresh timeout policy baselines.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DATA="${ROOT}/benchmarks/data"
CLI="${ROOT}/target/release/ontologos"
OUT_DIR="${ROOT}/benchmarks/data"
JSON_OUT="${OUT_DIR}/dl-perf-snapshot.json"
MD_OUT="${OUT_DIR}/dl-perf-snapshot.md"

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
  local start end elapsed
  start="$(python3 -c 'import time; print(time.perf_counter())')"
  "${CLI}" --profile "${profile}" --format json classify "${owl}" >/dev/null
  end="$(python3 -c 'import time; print(time.perf_counter())')"
  elapsed="$(python3 -c "print(round(${end} - ${start}, 3))")"
  echo "${corpus}|${profile}|${elapsed}"
}

echo "DL performance snapshot (release CLI)"

ROWS=()
while IFS= read -r row; do
  [[ -n "${row}" ]] && ROWS+=("${row}")
done < <(
  measure "family.owl" "dl"
  if [[ "${RUN_SLOW_DL_GATES:-0}" == "1" ]]; then
    measure "pizza.owl" "dl"
    measure "go-subset.owl" "dl"
  fi
)

python3 - "${JSON_OUT}" "${MD_OUT}" "${ROWS[@]}" <<'PY'
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

json_path = Path(sys.argv[1])
md_path = Path(sys.argv[2])
rows = [r.split("|", 2) for r in sys.argv[3:] if r]

targets = {
    "family.owl": {"target_s": 0.1, "label": "Family DL"},
    "pizza.owl": {"target_s": 30.0, "label": "Pizza DL (medium-DL ROADMAP)"},
    "go-subset.owl": {"target_s": 10.0, "label": "go-subset DL"},
}

results = []
for corpus, profile, elapsed_s in rows:
    elapsed = float(elapsed_s)
    meta = targets.get(corpus, {"target_s": None, "label": corpus})
    target = meta["target_s"]
    results.append(
        {
            "corpus": corpus,
            "profile": profile,
            "elapsed_s": elapsed,
            "target_s": target,
            "meets_target": (target is None) or (elapsed <= target),
        }
    )

doc = {
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "cli": "ontologos --profile dl classify (release)",
    "run_slow_dl_gates": bool(int(__import__("os").environ.get("RUN_SLOW_DL_GATES", "0"))),
    "results": results,
}
json_path.write_text(json.dumps(doc, indent=2) + "\n")

lines = [
    "# DL performance snapshot",
    "",
    f"Generated: {doc['generated_at']}",
    "",
    "| Corpus | Profile | Elapsed (s) | Target (s) | Meets target |",
    "|--------|---------|-------------|------------|--------------|",
]
for r in results:
    target = "" if r["target_s"] is None else f"{r['target_s']:.1f}"
    meets = "yes" if r["meets_target"] else "no"
    lines.append(
        f"| `{r['corpus']}` | {r['profile']} | {r['elapsed_s']:.3f} | {target} | {meets} |"
    )
lines.append("")
md_path.write_text("\n".join(lines))
print(f"wrote {json_path}")
print(f"wrote {md_path}")
for r in results:
    status = "ok" if r["meets_target"] else "SLOW"
    print(f"  {r['corpus']}: {r['elapsed_s']:.3f}s ({status})")
PY

echo "DL performance snapshot complete"
