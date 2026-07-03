#!/usr/bin/env bash
# Fail CI when user-facing docs drift from published vs workspace version channels,
# or when profile/version strings contradict release status.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

WORKSPACE_VERSION="$(grep -m1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')"
PUBLISHED_VERSION="1.0.0"
echo "Workspace version: ${WORKSPACE_VERSION}"
echo "Published version: ${PUBLISHED_VERSION}"

FAIL=0

# Files where Cargo.toml pins must match the published crates.io channel.
PUBLISHED_PIN_FILES=(
  README.md
  docs/getting-started/index.md
  docs/getting-started/classify-quickstart.md
  docs/getting-started/load-owl-file.md
  docs/getting-started/rdfs-materialization.md
  docs/getting-started/owl-rl-saturation.md
  docs/getting-started/owl-el-classification.md
  docs/guides/facade-api.md
  FAQ.md
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
DOCSRS_SCAN_FILES=(
  docs/getting-started
  docs/guides
  docs/reference
  docs/examples
  docs/comparison.md
  docs/security.md
  docs/architecture.md
  docs/index.md
  README.md
  FAQ.md
  mkdocs.yml
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
  find "${DOCSRS_SCAN_FILES[@]}" -type f \( -name '*.md' -o -name '*.yml' \) 2>/dev/null | sort
)

# Release-channel messaging on README and docs home (banner snippet or Install channels page).
for file in README.md docs/index.md; do
  has_banner=0
  if grep -q 'snippets/channel-banner.md' "$file" 2>/dev/null; then
    has_banner=1
  fi
  if grep -q 'install-channels' "$file" 2>/dev/null; then
    has_banner=1
  fi
  if grep -q "Latest tagged release is \*\*v1.0.0\*\*" "$file" 2>/dev/null; then
    has_banner=1
  fi
  if grep -q "Latest release is \*\*v1.0.0\*\*" "$file" 2>/dev/null; then
    has_banner=1
  fi
  if [[ "$has_banner" -eq 0 ]]; then
    echo "ERROR: ${file} missing release-channel messaging (banner snippet or install-channels link)"
    FAIL=1
  fi
  if ! grep -Eq 'v1\.0\.0.*(published|crates\.io|PyPI)' "$file"; then
    echo "ERROR: ${file} missing v1.0.0 published channel note"
    FAIL=1
  fi
done

# Canonical channel and limitations pages must exist.
for required in docs/guides/install-channels.md docs/guides/known-limitations.md docs/project/post-1.0-doc-update.md; do
  if [[ ! -f "$required" ]]; then
    echo "ERROR: missing required doc: ${required}"
    FAIL=1
  fi
done

# Profile stability matrix is the canonical stability surface.
if ! grep -q "profile-stability" docs/guides/preview-profiles.md 2>/dev/null; then
  echo "ERROR: preview-profiles.md must link to profile-stability.md"
  FAIL=1
fi
if grep -q "Stable (1.0)" docs/guides/preview-profiles.md docs/reference/cli.md 2>/dev/null; then
  echo "ERROR: contradictory DL 'Stable (1.0)' label still present"
  FAIL=1
fi

FORBIDDEN=(
  "not available in the CLI until v0.5"
  "CLI does **not** run OWL RL saturation"
  "default \"auto\" fails in v0.4"
  "tag pending"
  "Latest tagged release:** **v0.7.0"
  "Latest tagged release:** **v0.8.0"
  "Quick paths for upgrading to **v0.9.0**"
  "v0.9.0 on \`main\`"
  "not on PyPI 0.9.0"
  "PyPI **0.9.0**"
  "PyPI 0.9.0"
  "requires workspace **1.0.0** / \`main\`"
  "Requires build from \`main\`"
  "does not resolve \`owl:imports\`"
)
STALE_CHANNEL_SCAN=(
  docs/getting-started
  docs/guides
  docs/reference
  docs/examples
  docs/comparison.md
  docs/architecture.md
  docs/index.md
  docs/security.md
  README.md
  FAQ.md
  crates/ontologos-py/README.md
  .github/ISSUE_TEMPLATE
)
for phrase in "${FORBIDDEN[@]}"; do
  if grep -Rql --fixed-strings --exclude=check-doc-versions.sh \
    "$phrase" docs FAQ.md README.md CONTRIBUTING.md crates/ontologos-cli/src/main.rs 2>/dev/null; then
    echo "ERROR: forbidden stale phrase still present: ${phrase}"
    FAIL=1
  fi
done

# Stale dual-channel messaging outside migration/historical docs.
STALE_CHANNEL_PHRASES=(
  "not on PyPI 0.9.0"
  "PyPI **0.9.0**"
  "PyPI 0.9.0"
  "requires workspace **1.0.0** / \`main\`"
  "Requires build from \`main\`"
  "not available on PyPI **0.9.0**"
)
while IFS= read -r file; do
  case "$file" in
    docs/migration/*|docs/internal/*|docs/project/post-1.0-doc-update.md|docs/project/release-notes.md|docs/project/release-status.md|docs/project/spec.md)
      continue
      ;;
  esac
  for phrase in "${STALE_CHANNEL_PHRASES[@]}"; do
    if grep -Fq "$phrase" "$file" 2>/dev/null; then
      echo "ERROR: stale channel phrase in ${file}: ${phrase}"
      FAIL=1
    fi
  done
done < <(
  find "${STALE_CHANNEL_SCAN[@]}" -type f \( -name '*.md' -o -name '*.yml' \) 2>/dev/null | sort
)

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

# CLI after_help must match workspace version (built from main).
if ! grep -q "v${WORKSPACE_VERSION}:" crates/ontologos-cli/src/main.rs; then
  echo "ERROR: CLI after_help does not advertise v${WORKSPACE_VERSION}"
  FAIL=1
fi

# Migration hub must reference v1.0.0 upgrade path.
if ! grep -q "v0.9.x → v1.0.0" docs/migration/index.md; then
  echo "ERROR: migration hub missing v0.9.x → v1.0.0 path"
  FAIL=1
fi

if [[ "$FAIL" -ne 0 ]]; then
  exit 1
fi

echo "Documentation version check passed."
