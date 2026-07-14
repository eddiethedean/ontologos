#!/usr/bin/env bash
# Build and publish @ontologos/wasm to npm.
#
# Required env:
#   NPM_TOKEN — npm automation/publish token (GitHub secret)
#
# Optional env:
#   NPM_PACKAGE_VERSION — override package version (defaults to workspace
#     version, or the v*.*.* git tag when GITHUB_REF is a release tag)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

if [ -z "${NPM_TOKEN:-}" ]; then
  echo "Skipping npm WASM publish (NPM_TOKEN not set)"
  exit 0
fi

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "wasm-pack not found; install from https://rustwasm.github.io/wasm-pack/" >&2
  exit 1
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

echo "Building @ontologos/wasm (version ${version})..."
(
  cd crates/ontologos-wasm
  npm install
  npm run build
  npm version "${version}" --no-git-tag-version --allow-same-version
)

wasm_path="crates/ontologos-wasm/pkg/ontologos_wasm_bg.wasm"
if [ ! -f "${wasm_path}" ]; then
  echo "ERROR: expected ${wasm_path} after wasm-pack build" >&2
  exit 1
fi

# Authenticate with NPM_TOKEN. setup-node --registry-url points NPM_CONFIG_USERCONFIG
# at a temp npmrc that expands ${NODE_AUTH_TOKEN}; keep both in sync.
export NODE_AUTH_TOKEN="${NPM_TOKEN}"
npmrc="${NPM_CONFIG_USERCONFIG:-${HOME}/.npmrc}"
{
  echo "registry=https://registry.npmjs.org/"
  echo "//registry.npmjs.org/:_authToken=${NPM_TOKEN}"
  echo "always-auth=true"
} > "${npmrc}"

echo "Publishing @ontologos/wasm@${version} to npm (auth via NPM_TOKEN)..."
(
  cd crates/ontologos-wasm
  if output="$(npm publish --access public 2>&1)"; then
    echo "${output}"
    echo "Published @ontologos/wasm@${version}"
    exit 0
  fi
  echo "${output}"
  if grep -qiE 'cannot publish over|previously published|already (exists|published)|EPUBLISHCONFLICT' <<<"${output}"; then
    echo "@ontologos/wasm@${version} already on npm; skipping"
    exit 0
  fi
  echo "Failed to publish @ontologos/wasm@${version}" >&2
  exit 1
)
