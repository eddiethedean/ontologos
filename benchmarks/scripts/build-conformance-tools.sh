#!/usr/bin/env bash
# Build ontologos-conformance CLI tools in release mode (much faster parity scans).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT}"
TARGET_DIR="${CARGO_TARGET_DIR:-${ROOT}/target}"
cargo build --release -q -p ontologos-conformance --bins
echo "${TARGET_DIR}/release"
