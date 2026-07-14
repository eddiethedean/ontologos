#!/usr/bin/env bash
# Publish the Node native package `ontologos` to npm after CI collected
# multi-platform *.node artifacts into crates/ontologos-node/.
#
# Required env:
#   NPM_TOKEN — npm automation/publish token (GitHub secret)
#
# Optional env:
#   NPM_PACKAGE_VERSION — override package version (defaults to workspace
#     version, or the v*.*.* git tag when GITHUB_REF is a release tag)
#
# Expected layout before running (after download-artifact):
#   crates/ontologos-node/ontologos.*.node
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

if [ -z "${NPM_TOKEN:-}" ]; then
  echo "Skipping npm Node publish (NPM_TOKEN not set)"
  exit 0
fi

workspace_version="$(
  cargo metadata --format-version=1 --no-deps \
    | jq -r '.packages[] | select(.name == "ontologos-core") | .version'
)"

if [ -n "${NPM_PACKAGE_VERSION:-}" ]; then
  version="${NPM_PACKAGE_VERSION}"
elif [[ "${GITHUB_REF:-}" == refs/tags/v* ]]; then
  version="${GITHUB_REF#refs/tags/v}"
else
  version="${workspace_version}"
fi

if [ "${version}" != "${workspace_version}" ]; then
  echo "ERROR: publish version (${version}) does not match workspace (${workspace_version})" >&2
  exit 1
fi

cd crates/ontologos-node

shopt -s nullglob
nodes=(ontologos.*.node)
if [ "${#nodes[@]}" -eq 0 ]; then
  echo "ERROR: no ontologos.*.node artifacts found in crates/ontologos-node" >&2
  ls -la >&2 || true
  exit 1
fi

echo "Native binaries to publish:"
ls -la ontologos.*.node

# Require the five release targets declared in package.json napi.triples.
required=(
  ontologos.darwin-arm64.node
  ontologos.darwin-x64.node
  ontologos.linux-arm64-gnu.node
  ontologos.linux-x64-gnu.node
  ontologos.win32-x64-msvc.node
)
for f in "${required[@]}"; do
  if [ ! -f "${f}" ]; then
    echo "ERROR: missing required binary ${f}" >&2
    exit 1
  fi
done

npm version "${version}" --no-git-tag-version --allow-same-version

echo "Publishing ontologos@${version} to npm..."
echo "//registry.npmjs.org/:_authToken=${NPM_TOKEN}" > ~/.npmrc
if output="$(npm publish --access public 2>&1)"; then
  echo "${output}"
  echo "Published ontologos@${version}"
  exit 0
fi
echo "${output}"
if grep -qiE 'cannot publish over|previously published|already (exists|published)|EPUBLISHCONFLICT' <<<"${output}"; then
  echo "ontologos@${version} already on npm; skipping"
  exit 0
fi
echo "Failed to publish ontologos@${version}" >&2
exit 1
