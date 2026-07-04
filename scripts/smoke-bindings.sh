#!/usr/bin/env bash
# Run smoke tests for all language bindings (Java, .NET, C/C++).
# Prefer scripts/ci-bindings.sh for full build + test coverage.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export ONTOLOGOS_REPO_ROOT="$ROOT"

exec bash "$ROOT/scripts/ci-bindings.sh"
