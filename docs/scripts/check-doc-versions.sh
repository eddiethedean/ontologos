#!/usr/bin/env bash
# Fail CI when user-facing docs pin a crate version that differs from the workspace,
# or when profile/version strings drift across surfaces.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

WORKSPACE_VERSION="$(grep -m1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')"
echo "Workspace version: ${WORKSPACE_VERSION}"

FAIL=0

while IFS= read -r file; do
  while IFS= read -r line; do
    if [[ "$line" =~ ontologos-[a-z]+[[:space:]]*=[[:space:]]*\"([0-9]+\.[0-9]+\.[0-9]+)\" ]]; then
      pinned="${BASH_REMATCH[1]}"
      if [[ "$pinned" != "$WORKSPACE_VERSION" ]]; then
        echo "ERROR: ${file} pins ${pinned} (expected ${WORKSPACE_VERSION})"
        FAIL=1
      fi
    fi
    if [[ "$line" =~ docs\.rs/[^/]+/([0-9]+\.[0-9]+\.[0-9]+) ]]; then
      docsrs="${BASH_REMATCH[1]}"
      if [[ "$docsrs" != "$WORKSPACE_VERSION" ]]; then
        echo "ERROR: ${file} links docs.rs/${docsrs} (expected ${WORKSPACE_VERSION})"
        FAIL=1
      fi
    fi
  done < "$file"
done < <(
  find docs/getting-started docs/guides docs/reference docs/comparison.md docs/security.md docs/architecture.md FAQ.md \
    -type f -name '*.md' 2>/dev/null | sort
)

FORBIDDEN=(
  "not available in the CLI until v0.5"
  "CLI does **not** run OWL RL saturation"
  "default \"auto\" fails in v0.4"
  "tag pending"
  "Latest tagged release:** **v0.7.0"
  "Latest tagged release:** **v0.8.0"
  "v0.8.0: profile"
)
for phrase in "${FORBIDDEN[@]}"; do
  if grep -Rql --fixed-strings --exclude=check-doc-versions.sh \
    "$phrase" docs FAQ.md README.md crates/ontologos-cli/src/main.rs 2>/dev/null; then
    echo "ERROR: forbidden stale phrase still present: ${phrase}"
    FAIL=1
  fi
done

# Profile list must appear in canonical doc surfaces
PROFILE_MARKERS=( "dl-preview" "alc" "swrl" )
for marker in "${PROFILE_MARKERS[@]}"; do
  for file in docs/reference/cli.md docs/guides/python.md FAQ.md; do
    if ! grep -Fq "$marker" "$file" 2>/dev/null; then
      echo "ERROR: ${file} missing preview profile marker: ${marker}"
      FAIL=1
    fi
  done
done

# CLI after_help must match workspace version
if ! grep -q "v${WORKSPACE_VERSION}:" crates/ontologos-cli/src/main.rs; then
  echo "ERROR: CLI after_help does not advertise v${WORKSPACE_VERSION}"
  FAIL=1
fi

if [[ "$FAIL" -ne 0 ]]; then
  exit 1
fi

echo "Documentation version check passed."
