#!/usr/bin/env bash
# Build the MkDocs site with strict validation and no Material MkDocs 2.0 banner.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export NO_MKDOCS_2_WARNING=1

SITE_DIR="${1:-site}"
shift || true

exec mkdocs build --strict --site-dir "$SITE_DIR" "$@"
