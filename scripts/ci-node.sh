#!/usr/bin/env bash
# Build and test Node and WASM bindings (mirrors .github/workflows/ci.yml node job).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

run() {
  echo "==> $*"
  "$@"
}

run ./benchmarks/scripts/download.sh

run cargo test -p ontologos-js --locked

(
  cd crates/ontologos-node
  run npm install
  run npm run build
  run npm test
)

(
  cd crates/ontologos-wasm
  if command -v wasm-pack >/dev/null 2>&1; then
    run npm install
    run npm test
  else
    echo "wasm-pack not found; install from https://rustwasm.github.io/wasm-pack/" >&2
    exit 1
  fi
)

echo "Node/WASM checks passed"
