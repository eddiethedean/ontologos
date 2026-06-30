#!/usr/bin/env bash
# Head-to-head DL classification wall time: OntoLogos (release CLI) vs HermiT JAR.
#
# Prerequisites:
#   ./benchmarks/scripts/download.sh
#   ./benchmarks/scripts/download-hermit-jar.sh
#   export HERMIT_JAR=benchmarks/data/hermit.jar
#   java 11+ on PATH
#
# Optional:
#   RUN_SLOW_DL_GATES=1     include pizza.owl and go-subset.owl
#   HERMIT_PERF_ITERATIONS=3   repeat timed runs (median reported)
#   HERMIT_PERF_WARMUP=1    warmup before timing (default 1)
#
# Output:
#   benchmarks/data/hermit-perf-snapshot.json
#   benchmarks/data/hermit-perf-snapshot.md
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DATA="${ROOT}/benchmarks/data"
GOLDEN="${DATA}/dl-taxonomy-golden.json"
CLI="${ROOT}/target/release/ontologos"
HERMIT="${HERMIT_JAR:-}"
OUT_JSON="${DATA}/hermit-perf-snapshot.json"
OUT_MD="${DATA}/hermit-perf-snapshot.md"
ITERATIONS="${HERMIT_PERF_ITERATIONS:-1}"
WARMUP="${HERMIT_PERF_WARMUP:-1}"

if [[ -z "${HERMIT}" ]] || [[ ! -f "${HERMIT}" ]]; then
  echo "set HERMIT_JAR to a HermiT.jar path (see benchmarks/scripts/download-hermit-jar.sh)" >&2
  exit 1
fi
if ! command -v java >/dev/null 2>&1; then
  echo "java not found on PATH" >&2
  exit 1
fi
if [[ ! -f "${GOLDEN}" ]]; then
  echo "missing ${GOLDEN}" >&2
  exit 1
fi

if [[ ! -x "${CLI}" ]]; then
  "${ROOT}/benchmarks/scripts/build-release-cli.sh" >/dev/null
fi

now_s() {
  python3 -c 'import time; print(time.perf_counter())'
}

median() {
  python3 -c '
import statistics
import sys
vals = [float(x) for x in sys.argv[1:] if x]
print(round(statistics.median(vals), 3) if vals else "")
' "$@"
}

warmup_ontologos() {
  local owl="$1"
  local profile="$2"
  "${CLI}" --profile "${profile}" --format json classify "${owl}" >/dev/null
}

time_ontologos() {
  local owl="$1"
  local profile="$2"
  local start end
  start="$(now_s)"
  "${CLI}" --profile "${profile}" --format json classify "${owl}" >/dev/null
  end="$(now_s)"
  python3 -c "print(round(${end} - ${start}, 3))"
}

warmup_hermit() {
  local uri="$1"
  java -jar "${HERMIT}" -c "${uri}" >/dev/null 2>&1
}

time_hermit() {
  local uri="$1"
  local start end
  start="$(now_s)"
  if java -jar "${HERMIT}" -c "${uri}" >/dev/null 2>&1; then
    end="$(now_s)"
    python3 -c "print(round(${end} - ${start}, 3))"
    return 0
  fi
  echo ""
  return 1
}

measure_pair() {
  local corpus="$1"
  local profile="$2"
  local owl="${DATA}/${corpus}"
  if [[ ! -f "${owl}" ]]; then
    echo "skip|${corpus}|${profile}|missing corpus" >&2
    return 0
  fi

  local owl_abs uri
  owl_abs="$(cd "$(dirname "${owl}")" && pwd)/$(basename "${owl}")"
  uri="file://${owl_abs}"

  echo "benchmark: ${corpus} (profile=${profile})" >&2

  if [[ "${WARMUP}" == "1" ]]; then
    warmup_ontologos "${owl}" "${profile}" || true
    warmup_hermit "${uri}" || true
  fi

  local onto_times=() hermit_times=()
  local i hermit_ok=1 onto_elapsed hermit_elapsed

  for ((i = 0; i < ITERATIONS; i++)); do
    onto_elapsed="$(time_ontologos "${owl}" "${profile}")"
    onto_times+=("${onto_elapsed}")
    if hermit_elapsed="$(time_hermit "${uri}")"; then
      hermit_times+=("${hermit_elapsed}")
    else
      hermit_ok=0
      break
    fi
  done

  local onto_med hermit_med ratio
  onto_med="$(median "${onto_times[@]}")"
  if [[ "${hermit_ok}" -eq 1 ]]; then
    hermit_med="$(median "${hermit_times[@]}")"
    ratio="$(python3 -c "o,h=float('${onto_med}'),float('${hermit_med}'); print(round(o/h, 2) if h > 0 else '')")"
    echo "${corpus}|${profile}|${onto_med}|${hermit_med}|${ratio}|ok"
  else
    echo "${corpus}|${profile}|${onto_med}|| |hermit_failed"
  fi
}

ROWS=()
while IFS= read -r row; do
  [[ -n "${row}" ]] && ROWS+=("${row}")
done < <(
  while read -r corpus profile optional; do
    if [[ "${optional}" == "True" ]] && [[ "${RUN_SLOW_DL_GATES:-0}" != "1" ]]; then
      echo "skip optional corpus ${corpus} (set RUN_SLOW_DL_GATES=1)" >&2
      continue
    fi
    measure_pair "${corpus}" "${profile}"
  done < <(
    python3 - "${GOLDEN}" <<'PY'
import json
import sys

doc = json.load(open(sys.argv[1]))
for name, meta in doc["corpora"].items():
    if not meta.get("hermit_crosscheck", False):
        continue
    print(name, meta.get("profile", "dl"), meta.get("optional", False))
PY
  )
)

python3 - "${OUT_JSON}" "${OUT_MD}" "${ITERATIONS}" "${WARMUP}" "${HERMIT}" "${ROWS[@]}" <<'PY'
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

json_path = Path(sys.argv[1])
md_path = Path(sys.argv[2])
iterations = int(sys.argv[3])
warmup = sys.argv[4] == "1"
hermit_jar = sys.argv[5]
rows = [r.split("|", 5) for r in sys.argv[6:] if r]

results = []
for corpus, profile, onto_s, hermit_s, ratio_s, status in rows:
    entry = {
        "corpus": corpus,
        "profile": profile,
        "ontologos_s": float(onto_s) if onto_s else None,
        "hermit_s": float(hermit_s) if hermit_s.strip() else None,
        "ontologos_over_hermit": float(ratio_s) if ratio_s.strip() else None,
        "status": status,
    }
    results.append(entry)

doc = {
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ontologos": "ontologos --profile <profile> classify (release)",
    "hermit": f"java -jar {hermit_jar} -c <file-uri>",
    "iterations": iterations,
    "warmup": warmup,
    "run_slow_dl_gates": bool(int(__import__("os").environ.get("RUN_SLOW_DL_GATES", "0"))),
    "notes": [
        "HermiT may require network for owl:imports (e.g. family.owl swrla.owl).",
        "Ratios > 1 mean OntoLogos is slower than HermiT.",
    ],
    "results": results,
}
json_path.write_text(json.dumps(doc, indent=2) + "\n")

lines = [
    "# HermiT vs OntoLogos performance snapshot",
    "",
    f"Generated: {doc['generated_at']}",
    "",
    f"Iterations: {iterations} (median) · Warmup: {'yes' if warmup else 'no'}",
    "",
    "| Corpus | Profile | OntoLogos (s) | HermiT (s) | OntoLogos/HermiT | Status |",
    "|--------|---------|---------------|------------|------------------|--------|",
]
for r in results:
    onto = "" if r["ontologos_s"] is None else f"{r['ontologos_s']:.3f}"
    hermit = "" if r["hermit_s"] is None else f"{r['hermit_s']:.3f}"
    ratio = "" if r["ontologos_over_hermit"] is None else f"{r['ontologos_over_hermit']:.2f}x"
    lines.append(
        f"| `{r['corpus']}` | {r['profile']} | {onto} | {hermit} | {ratio} | {r['status']} |"
    )
lines.extend(
    [
        "",
        "HermiT may need network access to resolve `owl:imports` on some corpora.",
        "",
    ]
)
md_path.write_text("\n".join(lines))
print(f"wrote {json_path}")
print(f"wrote {md_path}")
for r in results:
    ratio = r["ontologos_over_hermit"]
    ratio_txt = f" ({ratio:.2f}x)" if ratio is not None else ""
    hermit = r["hermit_s"]
    hermit_txt = f" vs HermiT {hermit:.3f}s" if hermit is not None else ""
    print(f"  {r['corpus']}: OntoLogos {r['ontologos_s']:.3f}s{hermit_txt}{ratio_txt} [{r['status']}]")
PY

echo "HermiT performance comparison complete"
