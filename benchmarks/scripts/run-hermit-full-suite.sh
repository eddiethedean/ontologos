#!/usr/bin/env bash
# Run the full HermiT + OWL WG conformance suite (failure-first workflow).
# Blocking CI uses ONTOLOGOS_CI_PROMOTED_ONLY=1; this script runs every active test.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "${ROOT}"

export ONTOLOGOS_DL_BUDGET_SECS="${ONTOLOGOS_DL_BUDGET_SECS:-120}"
unset ONTOLOGOS_CI_PROMOTED_ONLY

THREADS="${ONTOLOGOS_TEST_THREADS:-8}"

echo "==> Full HermiT Java catalog"
cargo test -p ontologos-conformance --release --locked \
  --test hermit_generated \
  --test hermit_rdfs \
  --test hermit_rl \
  --test hermit_el \
  --test hermit_parser \
  -- \
  --test-threads="${THREADS}" \
  "$@"

echo "==> Full OWL WG catalog"
cargo test -p ontologos-conformance --release --locked \
  --test hermit_wg_generated \
  -- \
  --test-threads="${THREADS}" \
  "$@"
