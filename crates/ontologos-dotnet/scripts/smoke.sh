#!/usr/bin/env bash
# Smoke-test .NET bindings without the full solution runner.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
CSHARP_ROOT="$ROOT/crates/ontologos-dotnet/csharp"

echo "Building native library..."
cargo build -p ontologos-dotnet --release --manifest-path "$ROOT/Cargo.toml"

export ONTOLOGOS_REPO_ROOT="$ROOT"
dotnet test "$CSHARP_ROOT/Ontologos.sln" --configuration Release --verbosity minimal
