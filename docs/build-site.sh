#!/usr/bin/env bash
# Build the MkDocs site with strict validation and no Material MkDocs 2.0 banner.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export NO_MKDOCS_2_WARNING=1

if [[ -n "${READTHEDOCS:-}" ]]; then
  SITE_DIR="${READTHEDOCS_OUTPUT}/html"
  SKIP_DOC_SNIPPET_CARGO="${SKIP_DOC_SNIPPET_CARGO:-1}"
else
  SITE_DIR="${1:-site}"
  shift || true
fi

OUTPUT="$(
  mkdocs build --strict --site-dir "$SITE_DIR" "$@" 2>&1
)" || {
  echo "$OUTPUT"
  exit 1
}
echo "$OUTPUT"

if echo "$OUTPUT" | grep -qiE '(^|[[:space:]])WARNING[[:space:]-]|Warning from the Material|⚠[[:space:]]+Warning'; then
  echo "error: mkdocs build emitted warnings (see output above)" >&2
  exit 1
fi

if echo "$OUTPUT" | grep -q 'excluded from the built site'; then
  echo "error: mkdocs build has links to excluded documentation (see output above)" >&2
  exit 1
fi

chmod +x docs/scripts/check-doc-versions.sh
./docs/scripts/check-doc-versions.sh
chmod +x docs/scripts/check-doc-snippets.sh
if [[ "${SKIP_DOC_SNIPPET_CARGO:-}" == "1" ]]; then
  ./docs/scripts/check-doc-snippets.sh
else
  REQUIRE_DOC_SNIPPET_CARGO=1 ./docs/scripts/check-doc-snippets.sh
fi
