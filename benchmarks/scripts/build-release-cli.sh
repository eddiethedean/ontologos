#!/usr/bin/env bash
# Build the production (release) ontologos CLI used by perf gates and benchmarks.
# Profile: workspace [profile.release] (LTO + codegen-units=1).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CLI="${ROOT}/target/release/ontologos"

cd "${ROOT}"
cargo build -p ontologos-cli --release --locked "$@"

if [[ ! -x "${CLI}" ]]; then
  echo "release CLI missing after build: ${CLI}" >&2
  exit 1
fi

echo "${CLI}"
