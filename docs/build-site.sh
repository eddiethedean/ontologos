#!/usr/bin/env bash
# Build the MkDocs site with strict validation and no Material MkDocs 2.0 banner.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export NO_MKDOCS_2_WARNING=1

SITE_DIR="${1:-site}"
shift || true

OUTPUT="$(
  mkdocs build --strict --site-dir "$SITE_DIR" "$@" 2>&1
)" || {
  echo "$OUTPUT"
  exit 1
}
echo "$OUTPUT"

if echo "$OUTPUT" | grep -qiE '(^|[[:space:]])WARNING[[:space:]-]'; then
  echo "error: mkdocs build emitted warnings (see output above)" >&2
  exit 1
fi

chmod +x docs/scripts/check-doc-versions.sh
./docs/scripts/check-doc-versions.sh
chmod +x docs/scripts/check-doc-snippets.sh
./docs/scripts/check-doc-snippets.sh
