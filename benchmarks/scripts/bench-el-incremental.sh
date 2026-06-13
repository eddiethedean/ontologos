#!/usr/bin/env bash
set -euo pipefail

# Compare full vs incremental EL classify on a 10-axiom chain extension.
# Exit 0 if incremental is >=5x faster than full on this host.

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

cargo test -p ontologos-el --test incremental_bench -- --ignored --nocapture 2>&1
