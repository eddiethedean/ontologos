#!/usr/bin/env bash
# Fail CI when user-facing docs drift from published vs workspace version channels,
# or when profile/version strings contradict release status.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

WORKSPACE_VERSION="$(grep -m1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')"
PUBLISHED_VERSION="1.1.3"
echo "Workspace version: ${WORKSPACE_VERSION}"
echo "Published version: ${PUBLISHED_VERSION}"

FAIL=0

if [[ "$WORKSPACE_VERSION" != "$PUBLISHED_VERSION" ]]; then
  echo "ERROR: workspace version (${WORKSPACE_VERSION}) != published version (${PUBLISHED_VERSION})"
  FAIL=1
fi

# Files where Cargo.toml pins must match the published crates.io channel.
PUBLISHED_PIN_FILES=(
  README.md
  FAQ.md
  docs/getting-started/index.md
  docs/getting-started/classify-quickstart.md
  docs/getting-started/load-owl-file.md
  docs/getting-started/rdfs-materialization.md
  docs/getting-started/owl-rl-saturation.md
  docs/getting-started/owl-el-classification.md
  docs/getting-started/swrl.md
  docs/guides/facade-api.md
  docs/guides/rust-integration-contract.md
  docs/guides/install-channels.md
  crates/ontologos-core/README.md
  crates/ontologos-facade/README.md
  crates/ontologos-parser/README.md
  crates/ontologos-rl/README.md
  crates/ontologos-py/README.md
)

for file in "${PUBLISHED_PIN_FILES[@]}"; do
  if [[ ! -f "$file" ]]; then
    echo "ERROR: missing published-pin file: ${file}"
    FAIL=1
    continue
  fi
  while IFS= read -r line; do
    if [[ "$line" =~ ontologos-[a-z]+[[:space:]]*=[[:space:]]*\"([0-9]+\.[0-9]+\.[0-9]+)\" ]]; then
      pinned="${BASH_REMATCH[1]}"
      if [[ "$pinned" != "$PUBLISHED_VERSION" ]]; then
        echo "ERROR: ${file} pins ${pinned} in published install block (expected ${PUBLISHED_VERSION})"
        FAIL=1
      fi
    fi
  done < "$file"
done

# User-facing docs.rs links should point at the published crate docs.
DOCSRS_SCAN_DIRS=(
  docs/getting-started
  docs/guides
  docs/reference
  docs/examples
)
DOCSRS_SCAN_FILES=(
  docs/comparison.md
  docs/security.md
  docs/architecture.md
  docs/index.md
  README.md
  FAQ.md
  mkdocs.yml
  crates/ontologos-core/README.md
  crates/ontologos-facade/README.md
  crates/ontologos-parser/README.md
  crates/ontologos-rl/README.md
)

while IFS= read -r file; do
  while IFS= read -r line; do
    if [[ "$line" =~ docs\.rs/[^/]+/([0-9]+\.[0-9]+\.[0-9]+) ]]; then
      docsrs="${BASH_REMATCH[1]}"
      if [[ "$docsrs" != "$PUBLISHED_VERSION" ]]; then
        echo "ERROR: ${file} links docs.rs/${docsrs} (expected ${PUBLISHED_VERSION} for published API)"
        FAIL=1
      fi
    fi
  done < "$file"
done < <(
  find "${DOCSRS_SCAN_DIRS[@]}" "${DOCSRS_SCAN_FILES[@]}" -type f \( -name '*.md' -o -name '*.yml' \) 2>/dev/null \
    ! -path 'docs/migration/v0.*' ! -path 'docs/migration/v1.0.x-to-v1.1.0.md' | sort -u
)

# Release-status must document published channel.
if ! grep -q "${PUBLISHED_VERSION}" docs/project/release-status.md 2>/dev/null; then
  echo "ERROR: release-status.md missing published ${PUBLISHED_VERSION} channel"
  FAIL=1
fi

# Channel banner snippet must mention published version on registries.
if ! grep -q "Latest release is \*\*v${PUBLISHED_VERSION}\*\*" docs/snippets/channel-banner.md 2>/dev/null; then
  echo "ERROR: channel-banner.md missing Latest release v${PUBLISHED_VERSION}"
  FAIL=1
fi

# README and docs home must link install/release guidance.
for file in README.md docs/index.md; do
  has_banner=0
  if grep -q "snippets/channel-banner.md\|install-channels\|release-status\|v${PUBLISHED_VERSION}" "$file" 2>/dev/null; then
    has_banner=1
  fi
  if [[ "$has_banner" -eq 0 ]]; then
    echo "ERROR: ${file} missing release-channel messaging"
    FAIL=1
  fi
done

# PyPI / Python version alignment.
if ! grep -q "version = \"${PUBLISHED_VERSION}\"" crates/ontologos-py/pyproject.toml; then
  echo "ERROR: pyproject.toml version != ${PUBLISHED_VERSION}"
  FAIL=1
fi
if ! grep -q "__version__ = \"${PUBLISHED_VERSION}\"" crates/ontologos-py/python/ontologos/__init__.py; then
  echo "ERROR: ontologos __init__.py __version__ != ${PUBLISHED_VERSION}"
  FAIL=1
fi

FORBIDDEN=(
  "STAGED — NOT PUBLISHED"
  "not available in the CLI until v0.5"
  "CLI does **not** run OWL RL saturation"
  "default \"auto\" fails in v0.4"
  "Latest tagged release:** **v0.7.0"
  "Latest tagged release:** **v0.8.0"
  "Latest published:** **v1.0.0**"
  "For published v1.1.0 use 1.0.0 pins"
  "crates.io and PyPI remain **1.0.0**"
  "does not resolve \`owl:imports\`"
)
for phrase in "${FORBIDDEN[@]}"; do
  if grep -Rql --fixed-strings --exclude=check-doc-versions.sh \
    "$phrase" docs FAQ.md README.md CONTRIBUTING.md CHANGELOG.md crates/ontologos-cli/src/main.rs 2>/dev/null; then
    echo "ERROR: forbidden stale phrase still present: ${phrase}"
    FAIL=1
  fi
done

# Canonical channel and limitations pages must exist.
for required in \
  docs/guides/install-channels.md \
  docs/guides/known-limitations.md \
  docs/guides/before-you-integrate.md \
  docs/guides/bindings-overview.md \
  docs/project/release-status.md \
  docs/project/release-1.1-checklist.md \
  docs/snippets/channel-banner.md \
  docs/snippets/before-integrate-callout.md; do
  if [[ ! -f "$required" ]]; then
    echo "ERROR: missing required doc: ${required}"
    FAIL=1
  fi
done

if ! grep -q "profile-stability" docs/guides/preview-profiles.md 2>/dev/null; then
  echo "ERROR: preview-profiles.md must link to profile-stability.md"
  FAIL=1
fi

PROFILE_MARKERS=( "dl-preview" "alc" "swrl" )
for marker in "${PROFILE_MARKERS[@]}"; do
  for file in docs/reference/cli.md docs/guides/python.md FAQ.md; do
    if ! grep -Fq "$marker" "$file" 2>/dev/null; then
      echo "ERROR: ${file} missing preview profile marker: ${marker}"
      FAIL=1
    fi
  done
done

if ! grep -q "v${WORKSPACE_VERSION}:" crates/ontologos-cli/src/main.rs; then
  echo "ERROR: CLI after_help does not advertise v${WORKSPACE_VERSION}"
  FAIL=1
fi

if ! grep -q "v0.9.x → v1.0.0" docs/migration/index.md; then
  echo "ERROR: migration hub missing v0.9.x → v1.0.0 path"
  FAIL=1
fi
if ! grep -q "v1.0.x → v1.1.0" docs/migration/index.md; then
  echo "ERROR: migration hub missing v1.0.x → v1.1.0 path"
  FAIL=1
fi

for file in docs/getting-started/*.md; do
  if ! grep -q 'before-integrate-callout' "$file" 2>/dev/null; then
    echo "ERROR: ${file} missing before-integrate-callout snippet"
    FAIL=1
  fi
done

if [[ "$FAIL" -ne 0 ]]; then
  exit 1
fi

echo "Documentation version check passed."
