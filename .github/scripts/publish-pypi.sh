#!/usr/bin/env bash
set -euo pipefail

# Publish pre-built ontologos wheels/sdist to PyPI.
#
# Release CI builds wheels for Linux (x86_64, aarch64), macOS (aarch64, x86_64),
# Windows (x64, aarch64), plus sdist — then uploads via the publish-pypi job.
#
# This script is for manual uploads when you already have artifacts in dist/:
#   PYPI_API_TOKEN=pypi-... ./.github/scripts/publish-pypi.sh
#
# Local single-platform build (Linux only):
#   cd crates/ontologos-py && maturin build --release --sdist --out dist

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PY_CRATE="${ROOT}/crates/ontologos-py"
DIST="${PY_CRATE}/dist"

if [ -z "${PYPI_API_TOKEN:-}" ]; then
  echo "error: set PYPI_API_TOKEN to a PyPI API token with upload scope"
  exit 1
fi

if ! command -v maturin >/dev/null 2>&1; then
  echo "Installing maturin..."
  pip install 'maturin>=1.7,<2.0'
fi

if [ ! -d "${DIST}" ] || [ -z "$(ls -A "${DIST}" 2>/dev/null)" ]; then
  echo "Building ontologos sdist + local platform wheel (CI builds all OS/arch)..."
  cd "${PY_CRATE}"
  maturin build --release --sdist --out dist
else
  echo "Uploading existing artifacts in ${DIST}..."
fi

if [ "${1:-}" = "--dry-run" ]; then
  echo "Dry run complete:"
  ls -la "${DIST}"
  exit 0
fi

echo "Publishing ontologos to PyPI..."
MATURIN_PYPI_TOKEN="${PYPI_API_TOKEN}" maturin upload --skip-existing "${DIST}"/*

echo "Published ontologos to https://pypi.org/project/ontologos/"
