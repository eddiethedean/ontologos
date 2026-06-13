#!/usr/bin/env bash
# Serve the MkDocs site locally without the Material MkDocs 2.0 banner.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export NO_MKDOCS_2_WARNING=1

exec mkdocs serve "$@"
