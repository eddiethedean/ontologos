#!/usr/bin/env bash
# Download the standalone HermiT CLI JAR (not the Maven OWL API plugin bundle).
# Writes benchmarks/data/hermit.jar (gitignored) and verifies SHA-256.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DATA="${ROOT}/benchmarks/data"
DEST="${DATA}/hermit.jar"
CHECKSUMS="${ROOT}/benchmarks/checksums.sha256"
ZIP_URL="https://www.cs.ox.ac.uk/isg/tools/HermiT/download/current/HermiT.zip"

mkdir -p "${DATA}"

if [[ -f "${DEST}" ]]; then
  if grep -q "  hermit.jar$" "${CHECKSUMS}" 2>/dev/null; then
    expected="$(grep "  hermit.jar$" "${CHECKSUMS}" | awk '{print $1}')"
    actual="$(shasum -a 256 "${DEST}" | awk '{print $1}')"
    if [[ "${actual}" == "${expected}" ]]; then
      echo "hermit.jar already present (checksum ok)"
      echo "export HERMIT_JAR=\"${DEST}\""
      exit 0
    fi
    echo "existing hermit.jar checksum mismatch — re-downloading" >&2
  fi
fi

TMP_ZIP="$(mktemp "${TMPDIR:-/tmp}/hermit-download.XXXXXX.zip")"
trap 'rm -f "${TMP_ZIP}"' EXIT

echo "Downloading HermiT release from ${ZIP_URL}"
curl -fsSL -L "${ZIP_URL}" -o "${TMP_ZIP}"
unzip -p "${TMP_ZIP}" HermiT.jar >"${DEST}"

if grep -q "  hermit.jar$" "${CHECKSUMS}" 2>/dev/null; then
  expected="$(grep "  hermit.jar$" "${CHECKSUMS}" | awk '{print $1}')"
  actual="$(shasum -a 256 "${DEST}" | awk '{print $1}')"
  if [[ "${actual}" != "${expected}" ]]; then
    echo "checksum mismatch for hermit.jar" >&2
    echo "  expected: ${expected}" >&2
    echo "  actual:   ${actual}" >&2
    exit 1
  fi
  echo "checksum ok: hermit.jar"
else
  echo "warning: no hermit.jar entry in ${CHECKSUMS}; skipping verify" >&2
fi

echo "HermiT JAR ready at ${DEST}"
echo "export HERMIT_JAR=\"${DEST}\""
