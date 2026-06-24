#!/usr/bin/env bash
# Fail CI when generated HermiT catalog artifacts drift from generator output.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT}"

if [[ ! -d HermiT/src/test/java/org/semanticweb/HermiT ]]; then
  echo "HermiT checkout missing — refreshing from vendored catalog (--activate-all-from-disk)"
  python3 tests/hermit/generate_catalog.py --activate-all-from-disk
else
  python3 tests/hermit/generate_catalog.py --activate-all-from-disk
fi

if ! git diff --quiet -- benchmarks/data/hermit/catalog/cases.json \
  benchmarks/data/hermit/catalog/wg_cases.json \
  crates/ontologos-conformance/tests/hermit_generated.rs \
  crates/ontologos-conformance/tests/hermit_wg_generated.rs; then
  echo "HermiT catalog drift detected. Run: python3 tests/hermit/generate_catalog.py" >&2
  git diff --stat -- benchmarks/data/hermit/catalog crates/ontologos-conformance/tests/hermit_generated.rs crates/ontologos-conformance/tests/hermit_wg_generated.rs >&2 || true
  exit 1
fi

echo "HermiT catalog up to date"
