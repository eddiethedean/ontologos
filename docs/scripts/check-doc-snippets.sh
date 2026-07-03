#!/usr/bin/env bash
# Fail CI when user-facing docs reference removed APIs or broken internal links.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

FAIL=0

# User-facing doc roots (exclude historical migration and internal maintainer docs).
USER_DOC_DIRS=(
  docs/getting-started
  docs/guides
  docs/reference
  docs/examples
  docs/comparison.md
  docs/index.md
  README.md
  FAQ.md
  crates/ontologos-py/README.md
  crates/ontologos-facade/README.md
  crates/ontologos-rl/README.md
  crates/ontologos-rl/README.md
)

FORBIDDEN_API=(
  classify_with_profile
  QueryEngine
  hermit-parity-honest-assessment.md
)

for pattern in "${FORBIDDEN_API[@]}"; do
  while IFS= read -r file; do
    echo "ERROR: ${file} references removed or internal-only symbol: ${pattern}"
    FAIL=1
  done < <(
    find "${USER_DOC_DIRS[@]}" -type f \( -name '*.md' \) 2>/dev/null \
      | xargs grep -l --fixed-strings "$pattern" 2>/dev/null || true
  )
done

# Channel banner snippet must exist and be included in key pages.
SNIPPET="docs/snippets/channel-banner.md"
if [[ ! -f "$SNIPPET" ]]; then
  echo "ERROR: missing ${SNIPPET}"
  FAIL=1
fi

BANNER_PAGES=(
  docs/index.md
  docs/getting-started/index.md
  docs/guides/install-channels.md
  docs/project/release-status.md
  docs/migration/index.md
)

for file in "${BANNER_PAGES[@]}"; do
  if ! grep -q 'snippets/channel-banner.md' "$file" 2>/dev/null; then
    echo "ERROR: ${file} missing channel banner include"
    FAIL=1
  fi
done

# Public evaluator scope and adoption pages must exist.
for required in docs/guides/evaluator-scope.md docs/guides/install-channels.md docs/guides/known-limitations.md; do
  if [[ ! -f "$required" ]]; then
    echo "ERROR: missing ${required}"
    FAIL=1
  fi
done

# Compile-check canonical facade documentation pattern (requires Rust toolchain).
if [[ "${SKIP_DOC_SNIPPET_CARGO:-}" == "1" ]]; then
  echo "Skipping facade doc snippet compile check (SKIP_DOC_SNIPPET_CARGO=1)"
elif ! command -v cargo >/dev/null 2>&1; then
  if [[ "${REQUIRE_DOC_SNIPPET_CARGO:-}" == "1" ]]; then
    echo "ERROR: cargo required for facade doc snippet compile check"
    FAIL=1
  else
    echo "Skipping facade doc snippet compile check (cargo not installed)"
  fi
else
  echo "Running facade doc snippet compile check..."
  cargo test -p ontologos-facade --test doc_snippets --locked --quiet
fi

if [[ "$FAIL" -ne 0 ]]; then
  exit 1
fi

echo "Documentation snippet check passed."
