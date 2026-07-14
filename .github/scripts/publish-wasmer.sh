#!/usr/bin/env bash
# Build ontologos-wasm with wasm-pack and publish to the Wasmer Registry.
# Package: eddiethedean/ontologos
#
# Required env:
#   WASMER_TOKEN — registry API token (GitHub secret)
#
# Optional env:
#   WASMER_PACKAGE_VERSION — override package version (defaults to workspace
#     version, or the v*.*.* git tag when GITHUB_REF is a release tag)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

if [ -z "${WASMER_TOKEN:-}" ]; then
  echo "Skipping Wasmer publish (WASMER_TOKEN not set)"
  exit 0
fi

if ! command -v wasmer >/dev/null 2>&1; then
  echo "wasmer CLI not found on PATH" >&2
  exit 1
fi

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "wasm-pack not found; install from https://rustwasm.github.io/wasm-pack/" >&2
  exit 1
fi

workspace_version="$(
  cargo metadata --format-version=1 --no-deps \
    | jq -r '.packages[] | select(.name == "ontologos-core") | .version'
)"

if [ -n "${WASMER_PACKAGE_VERSION:-}" ]; then
  version="${WASMER_PACKAGE_VERSION}"
elif [[ "${GITHUB_REF:-}" == refs/tags/v* ]]; then
  version="${GITHUB_REF#refs/tags/v}"
else
  version="${workspace_version}"
fi

if [ "${version}" != "${workspace_version}" ]; then
  echo "ERROR: publish version (${version}) does not match workspace (${workspace_version})" >&2
  exit 1
fi

echo "Building WASM package (version ${version})..."
(
  cd crates/ontologos-wasm
  npm install
  npm run build
)

wasm_path="crates/ontologos-wasm/pkg/ontologos_wasm_bg.wasm"
if [ ! -f "${wasm_path}" ]; then
  echo "ERROR: expected ${wasm_path} after wasm-pack build" >&2
  exit 1
fi

echo "Publishing eddiethedean/ontologos@${version} to Wasmer Registry..."
(
  cd crates/ontologos-wasm
  # --non-interactive: no prompts in CI
  # --version: keep wasmer.toml in sync with workspace without editing the file
  if output="$(wasmer publish --non-interactive --version "${version}" 2>&1)"; then
    echo "${output}"
    echo "Published eddiethedean/ontologos@${version}"
    exit 0
  fi
  echo "${output}"
  if grep -qiE 'already (exists|published|uploaded)|version .* already' <<<"${output}"; then
    echo "eddiethedean/ontologos@${version} already on Wasmer Registry; skipping"
    exit 0
  fi
  echo "Failed to publish eddiethedean/ontologos@${version}" >&2
  exit 1
)
