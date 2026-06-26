#!/usr/bin/env bash
# HermiT Tier B classification corpora gate.
# Classifies vendored ClassificationTest XML fixtures with in-house EL and compares
# to HermiT hierarchy goldens (.txt). Complements compare-pizza-el-golden.sh, which
# checks owlcs pizza.owl against a committed JSON baseline.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
HERMIT_RES="${ROOT}/benchmarks/data/hermit/reasoner/res"

FIXTURES=(
  "pizza.xml"
  "wine.xml"
  "galen-ians-full-undoctored.xml"
  "propreo.xml"
)

echo "HermiT Tier B classification fixtures"

for fixture in "${FIXTURES[@]}"; do
  xml="${HERMIT_RES}/${fixture}"
  golden="${HERMIT_RES}/${fixture}.txt"
  if [[ ! -f "${xml}" ]]; then
    echo "missing fixture: ${xml}" >&2
    exit 1
  fi
  if [[ ! -f "${golden}" ]]; then
    echo "missing golden: ${golden}" >&2
    exit 1
  fi
  pairs="$(grep -c ' SubClassOf ' "${golden}" || true)"
  echo "  ${fixture}: ${pairs} golden subsumptions"
done

cargo test -p ontologos-conformance --release --locked --test hermit_el

echo "Tier B classification fixtures: all checks passed"
