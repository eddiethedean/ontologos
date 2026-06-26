#!/usr/bin/env bash
# Tier C reference harness: compare OntoLogos taxonomies to vendored golden baselines.
# Optional: set KONCLUDE_BIN or HERMIT_JAR for external cross-checks (not required for CI).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DATA="${ROOT}/benchmarks/data"

echo "OntoLogos Tier C harness"

# Tier B: Pizza EL golden (in-house EL baseline).
if [[ -f "${DATA}/pizza-el-golden.json" ]]; then
  "${ROOT}/benchmarks/scripts/compare-pizza-el-golden.sh"
fi

# Tier C: DL taxonomy goldens (family.owl + documented tolerance).
if [[ -f "${DATA}/dl-taxonomy-golden.json" ]]; then
  chmod +x "${ROOT}/benchmarks/scripts/compare-dl-taxonomy.sh"
  "${ROOT}/benchmarks/scripts/compare-dl-taxonomy.sh"
fi

# Hybrid / corpora smoke: load + profile detect on OBO subset (no full DL classify).
if [[ -f "${DATA}/go-subset.owl" ]]; then
  echo "corpus smoke: profile detect on go-subset.owl"
  cargo run -q -p ontologos-cli --release -- profile "${DATA}/go-subset.owl" 2>/dev/null | head -3 || \
    cargo run -q -p ontologos-cli -- profile "${DATA}/go-subset.owl" | head -3
fi

if [[ -f "${DATA}/hermit/reasoner/res/pizza.xml" ]]; then
  chmod +x "${ROOT}/benchmarks/scripts/compare-classification-fixtures.sh"
  "${ROOT}/benchmarks/scripts/compare-classification-fixtures.sh"
fi

if [[ -n "${KONCLUDE_BIN:-}" ]] && command -v "${KONCLUDE_BIN}" >/dev/null; then
  echo "Konclude found at ${KONCLUDE_BIN} (optional cross-check)"
  "${ROOT}/benchmarks/scripts/run-reference-baseline.sh" || true
fi

if [[ -n "${HERMIT_JAR:-}" ]] && [[ -f "${HERMIT_JAR}" ]]; then
  echo "HermiT JAR found at ${HERMIT_JAR} — running optional cross-check"
  chmod +x "${ROOT}/benchmarks/scripts/compare-dl-hermit-crosscheck.sh"
  "${ROOT}/benchmarks/scripts/compare-dl-hermit-crosscheck.sh" || true
fi

echo "Tier C harness: all gated checks passed"
