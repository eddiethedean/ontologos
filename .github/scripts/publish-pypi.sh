#!/usr/bin/env bash
set -euo pipefail

# Publish the ontologos PyPI package (maturin).
# Requires PYPI_API_TOKEN (pypi token with upload scope).
#
# Usage:
#   PYPI_API_TOKEN=pypi-... ./.github/scripts/publish-pypi.sh
#   PYPI_API_TOKEN=pypi-... ./.github/scripts/publish-pypi.sh --dry-run

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PY_CRATE="${ROOT}/crates/ontologos-py"

if [ -z "${PYPI_API_TOKEN:-}" ]; then
  echo "error: set PYPI_API_TOKEN to a PyPI API token with upload scope"
  exit 1
fi

cd "${PY_CRATE}"

if ! command -v maturin >/dev/null 2>&1; then
  echo "Installing maturin..."
  pip install 'maturin>=1.7,<2.0'
fi

echo "Building ontologos Python package (wheel + sdist)..."
maturin build --release --sdist --out dist

if [ "${1:-}" = "--dry-run" ]; then
  echo "Dry run complete. Wheels and sdist in ${PY_CRATE}/dist"
  ls -la dist
  exit 0
fi

echo "Publishing ontologos to PyPI..."
MATURIN_PYPI_TOKEN="${PYPI_API_TOKEN}" maturin upload --skip-existing dist/*

echo "Published ontologos to https://pypi.org/project/ontologos/"
