#!/usr/bin/env bash
# Generate a trimmed GO extract for EL CI benchmarks.
# Requires ROBOT (https://github.com/ontodev/robot) on PATH and benchmarks/data/go.owl.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
GO_FULL="${ROOT}/benchmarks/data/go.owl"
GO_SUBSET="${ROOT}/benchmarks/data/go-subset.owl"

if ! command -v robot >/dev/null 2>&1; then
  echo "robot not found; skip go-subset generation (install ROBOT or vendor go-subset.owl)"
  exit 0
fi

if [[ ! -f "${GO_FULL}" ]]; then
  echo "missing ${GO_FULL}; download full GO first"
  exit 1
fi

robot filter --input "${GO_FULL}" --select "GO_0008150" --output "${GO_SUBSET}"
echo "Wrote ${GO_SUBSET}"
