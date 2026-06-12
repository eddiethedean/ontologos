#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DATA="${ROOT}/benchmarks/data"
CHECKSUMS="${ROOT}/benchmarks/checksums.sha256"

mkdir -p "${DATA}"

verify_checksum() {
  local file="$1"
  local name
  name="$(basename "${file}")"
  if [[ ! -f "${CHECKSUMS}" ]]; then
    echo "warning: ${CHECKSUMS} missing; skipping checksum for ${name}" >&2
    return 0
  fi
  local expected
  expected="$(grep "  ${name}$" "${CHECKSUMS}" | awk '{print $1}' || true)"
  if [[ -z "${expected}" ]]; then
    echo "warning: no checksum entry for ${name} in ${CHECKSUMS}" >&2
    return 0
  fi
  local actual
  actual="$(shasum -a 256 "${file}" | awk '{print $1}')"
  if [[ "${actual}" != "${expected}" ]]; then
    echo "checksum mismatch for ${name}" >&2
    echo "  expected: ${expected}" >&2
    echo "  actual:   ${actual}" >&2
    return 1
  fi
  echo "checksum ok: ${name}"
}

download() {
  local url="$1"
  local dest="$2"
  echo "Downloading ${dest}..."
  curl -fsSL "${url}" -o "${dest}"
}

# Pizza — EL tutorial corpus (owlcs/pizza-ontology, RDF/XML)
PIZZA_URL="https://github.com/owlcs/pizza-ontology/raw/refs/heads/master/pizza.owl"
download "${PIZZA_URL}" "${DATA}/pizza.owl"
verify_checksum "${DATA}/pizza.owl"

# Family — classic Protégé/rexster family ontology (RL smoke test).
# Vendored in-repo at benchmarks/data/family.owl; refresh with --update-family.
FAMILY_URL="https://github.com/martinhbramwell/Monetary-Ontology-Walkabout/raw/master/rexster/extension/example/src/main/resources/data/family.swrl.owl"

if [[ "${1:-}" == "--update-family" ]]; then
  download "${FAMILY_URL}" "${DATA}/family.owl"
  verify_checksum "${DATA}/family.owl"
elif [[ -f "${DATA}/family.owl" ]]; then
  verify_checksum "${DATA}/family.owl"
else
  echo "family.owl not found; vendored copy should live at benchmarks/data/family.owl"
  echo "  or run: $0 --update-family"
  exit 1
fi

# Vendored EL perf + golden baselines (committed in-repo).
for vendored in go-subset.owl pizza-el-golden.json; do
  if [[ -f "${DATA}/${vendored}" ]]; then
    verify_checksum "${DATA}/${vendored}"
  else
    echo "missing vendored ${vendored} at ${DATA}/${vendored}" >&2
    exit 1
  fi
done

echo "Done. Corpus files in ${DATA}"
