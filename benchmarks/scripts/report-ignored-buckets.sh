#!/usr/bin/env bash
# Bucket dormant #[ignore] conformance tests by catalog ignore_reason (Tier B4 triage).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CASES="${ROOT}/benchmarks/data/hermit/catalog/cases.json"
OUT="${ROOT}/benchmarks/data/ignored-buckets.json"

python3 - "${CASES}" "${OUT}" <<'PY'
import json
import sys
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path

cases_path = Path(sys.argv[1])
out_path = Path(sys.argv[2])
cases = json.loads(cases_path.read_text())

ignored_statuses = {"excluded", "internal", "planned", "migrated"}
buckets: Counter[str] = Counter()
by_status: Counter[str] = Counter()
ids_by_bucket: dict[str, list[str]] = {}

for case in cases:
    status = case.get("status", "")
    if status == "covered":
        continue
    if status not in ignored_statuses:
        continue
    reason = case.get("ignore_reason") or status
    buckets[reason] += 1
    by_status[status] += 1
    ids_by_bucket.setdefault(reason, []).append(case["id"])

doc = {
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "total_activatable": sum(buckets.values()),
    "by_status": dict(sorted(by_status.items())),
    "by_reason": dict(sorted(buckets.items(), key=lambda kv: (-kv[1], kv[0]))),
    "sample_ids": {k: v[:5] for k, v in sorted(ids_by_bucket.items())},
}
out_path.parent.mkdir(parents=True, exist_ok=True)
out_path.write_text(json.dumps(doc, indent=2) + "\n")
print(json.dumps(doc, indent=2))
PY
