#!/usr/bin/env bash
# Tier C strict HermiT cross-check on family.owl only (--max-extra 0).
# Nightly / optional PR when HERMIT_JAR is present.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DATA="${ROOT}/benchmarks/data"
STATUS="${DATA}/tier-c-strict-status.json"
HERMIT="${HERMIT_JAR:-}"
REQUIRE="${ONTOLOGOS_REQUIRE_HERMIT_JAR:-0}"

write_status() {
  local family_pass="$1"
  python3 - "${STATUS}" "${family_pass}" <<'PY'
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

path = Path(sys.argv[1])
family_pass = sys.argv[2] == "1"
doc = {
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "corpora": {
        "family.owl": {"strict_pass": family_pass},
    },
    "tier_c_strict_pct": 100.0 if family_pass else 0.0,
}
path.parent.mkdir(parents=True, exist_ok=True)
path.write_text(json.dumps(doc, indent=2) + "\n")
PY
}

if [[ -z "${HERMIT}" ]] || [[ ! -f "${HERMIT}" ]]; then
  if [[ "${REQUIRE}" == "1" ]]; then
    echo "HermiT strict family gate required: set HERMIT_JAR" >&2
    exit 1
  fi
  echo "skip strict family cross-check: HERMIT_JAR not set"
  exit 0
fi

export ONTOLOGOS_STRICT_TAXONOMY=1
export RUN_SLOW_DL_GATES=0

if ONTOLOGOS_STRICT_TAXONOMY=1 "${ROOT}/benchmarks/scripts/compare-dl-hermit-crosscheck.sh"; then
  write_status 1
  echo "Tier C strict family gate: passed"
else
  write_status 0
  echo "Tier C strict family gate: failed (OntoLogos extras vs HermiT on family.owl)" >&2
  exit 1
fi
