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
  if [[ "${RUNNER_OS:-}" == "Windows" ]]; then
    echo "checksum skip (Windows line endings): ${name}"
    return 0
  fi
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
  if command -v shasum >/dev/null 2>&1; then
    actual="$(shasum -a 256 "${file}" | awk '{print $1}')"
  elif command -v sha256sum >/dev/null 2>&1; then
    actual="$(sha256sum "${file}" | awk '{print $1}')"
  else
    actual="$(openssl dgst -sha256 "${file}" | awk '{print $NF}')"
  fi
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
  if [[ -f "${dest}" ]] && verify_checksum "${dest}"; then
    return 0
  fi
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

# HermiT ClassificationTest fixtures (Tier B — vendored under benchmarks/data/hermit/).
HERMIT_RES="${DATA}/hermit/reasoner/res"
for fixture in pizza.xml pizza.xml.txt wine.xml wine.xml.txt \
  galen-ians-full-undoctored.xml galen-ians-full-undoctored.xml.txt \
  propreo.xml propreo.xml.txt; do
  if [[ ! -f "${HERMIT_RES}/${fixture}" ]]; then
    echo "missing vendored ClassificationTest fixture: ${HERMIT_RES}/${fixture}" >&2
    exit 1
  fi
done
echo "classification fixtures ok: pizza, wine, galen, propreo"

echo "Done. Corpus files in ${DATA}"
