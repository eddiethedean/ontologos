#!/usr/bin/env bash
# Triage planned HermiT Java + OWL WG catalog cases (engine gap vs missing assertions vs promotion).
#
# Parallelism: ONTOLOGOS_DL_MAX_WORKERS, ONTOLOGOS_SCAN_THREADS (see promote-hermit-catalog.sh).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT="${ROOT}/docs/internal/planned-backlog-triage.json"
MD="${ROOT}/docs/internal/planned-backlog-triage.md"

cd "${ROOT}"
BIN="$("${ROOT}/benchmarks/scripts/build-conformance-tools.sh")"
PLANNED_BACKLOG_OUT="${OUT}" "${BIN}/audit_planned_backlog" >/dev/null

python3 - "${OUT}" "${MD}" <<'PY'
import json
import sys
from collections import Counter
from datetime import datetime, timezone

src, md_path = sys.argv[1], sys.argv[2]
audit = json.load(open(src))
summary = audit["summary"]
now = datetime.now(timezone.utc).strftime("%Y-%m-%d")

lines = [
    f"# Planned backlog triage",
    "",
    f"**Generated:** {now} (UTC) via `benchmarks/scripts/audit-planned-backlog.sh`",
    "",
    "Do not edit by hand — regenerate after catalog or engine changes.",
    "",
    "## Summary",
    "",
    f"| Catalog | Planned |",
    f"|---------|--------:|",
    f"| HermiT Java (`cases.json`) | {summary['java_total']} |",
    f"| OWL WG (`wg_cases.json`) | {summary['wg_total']} |",
    "",
    "### Java by category",
    "",
    "| Category | Count |",
    "|----------|------:|",
]
for k, v in sorted(summary["java_by_category"].items()):
    lines.append(f"| `{k}` | {v} |")
lines += [
    "",
    "### WG by category",
    "",
    "| Category | Count |",
    "|----------|------:|",
]
for k, v in sorted(summary["wg_by_category"].items()):
    lines.append(f"| `{k}` | {v} |")

promo_java = [c for c in audit["java"] if c["category"] == "promotion_candidate"]
gap_java = [c for c in audit["java"] if c["category"] == "engine_gap"]
lines += [
    "",
    "## Promotion candidates (Java)",
    "",
]
if promo_java:
    for c in promo_java[:20]:
        lines.append(f"- `{c['id']}` ({c['engine']})")
    if len(promo_java) > 20:
        lines.append(f"- … and {len(promo_java) - 20} more (see JSON)")
else:
    lines.append("_None — run `promote_catalog` after engine fixes._")

lines += ["", "## Engine gaps (sample Java)", ""]
for c in gap_java[:15]:
    detail = (c.get("detail") or "")[:120]
    lines.append(f"- `{c['id']}` — {detail}")
if len(gap_java) > 15:
    lines.append(f"- … and {len(gap_java) - 15} more (see JSON)")

open(md_path, "w").write("\n".join(lines) + "\n")
print("wrote", md_path)
PY

echo "Triage written to ${OUT} and ${MD}"
