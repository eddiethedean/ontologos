#!/usr/bin/env bash
# Build and test Java, .NET, and C/C++ bindings (mirrors .github/workflows/ci.yml bindings job).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export ONTOLOGOS_REPO_ROOT="$ROOT"

run() {
  echo "==> $*"
  "$@"
}

run cargo build -p ontologos-jni -p ontologos-dotnet -p ontologos-c --release --locked

run cargo test -p ontologos-js --locked

if command -v mvn >/dev/null 2>&1; then
  run mvn -f "$ROOT/crates/ontologos-java/java/pom.xml" test --batch-mode
else
  run bash "$ROOT/crates/ontologos-java/scripts/smoke.sh"
fi

if command -v dotnet >/dev/null 2>&1; then
  run dotnet test "$ROOT/crates/ontologos-dotnet/csharp/Ontologos.sln" --configuration Release
else
  echo "dotnet not found; skipping .NET tests" >&2
  exit 1
fi

run bash "$ROOT/crates/ontologos-c/scripts/smoke.sh"

echo "Binding checks passed"
