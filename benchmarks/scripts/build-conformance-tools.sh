#!/usr/bin/env bash
# Build ontologos-conformance CLI tools in release mode (much faster parity scans).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT}"
TARGET_DIR="${CARGO_TARGET_DIR:-${ROOT}/target}"
BIN_DIR="${TARGET_DIR}/release"

needs_build=0
for bin in parity_status wg_failures promote_catalog promote_wg sync_promoted audit_planned_backlog engine_failures; do
  if [[ ! -x "${BIN_DIR}/${bin}" ]]; then
    needs_build=1
    break
  fi
done
if [[ "${needs_build}" -eq 0 ]] && [[ "${BIN_DIR}/parity_status" -ot "${ROOT}/crates/ontologos-conformance/src/catalog.rs" ]]; then
  needs_build=1
fi

if [[ "${needs_build}" -eq 1 ]]; then
  cargo build --release -q -p ontologos-conformance --bins
fi
echo "${BIN_DIR}"
