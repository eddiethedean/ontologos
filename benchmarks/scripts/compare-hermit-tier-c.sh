#!/usr/bin/env bash
# Tier C reference harness: compare OntoLogos DL taxonomy to vendored golden baselines.
# Optional: set KONCLUDE_BIN or HERMIT_JAR for external cross-checks (not required for CI).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DATA="${ROOT}/benchmarks/data"
GOLDEN="${DATA}/dl-family-golden.json"

echo "OntoLogos Tier C smoke (Pizza EL golden already gated separately)"

if [[ -f "${DATA}/pizza-el-golden.json" ]]; then
  "${ROOT}/benchmarks/scripts/compare-pizza-el-golden.sh"
fi

# DL smoke: classify pizza.owl (DL-detected corpus) and compare subsumption count to golden baseline.
if [[ -f "${DATA}/pizza.owl" ]]; then
  echo "DL smoke: classifying pizza.owl"
  OUT="$(mktemp)"
  cargo run -q -p ontologos-cli --release -- --profile dl --format json classify "${DATA}/pizza.owl" >"${OUT}" 2>/dev/null || \
    cargo run -q -p ontologos-cli -- --profile dl --format json classify "${DATA}/pizza.owl" >"${OUT}"
  SUBS="$(python3 -c "import json,sys; d=json.load(open(sys.argv[1])); print(d.get('subsumption_count', len(d.get('subsumptions', []))))" "${OUT}")"
  echo "pizza.owl DL subsumptions: ${SUBS}"
  if [[ -f "${GOLDEN}" ]]; then
    EXPECT="$(python3 -c "import json; g=json.load(open('${GOLDEN}')); print(g.get('subsumption_count', -1))")"
    if [[ "${EXPECT}" -gt 0 ]]; then
      if [[ "${SUBS}" != "${EXPECT}" ]]; then
        echo "DL golden mismatch: expected ${EXPECT}, got ${SUBS}" >&2
        rm -f "${OUT}"
        exit 1
      fi
      echo "DL golden match (${SUBS} subsumptions)"
    else
      echo "note: DL golden baseline not pinned (subsumption_count=${EXPECT}); logged ${SUBS}"
    fi
  else
    echo "note: no ${GOLDEN}; skipping DL subsumption count check"
  fi
  rm -f "${OUT}"
fi

if [[ -f "${DATA}/hermit/reasoner/res/pizza.xml" ]]; then
  echo "HermiT pizza fixture present — run: cargo test -p ontologos-conformance hermit_classification_pizza"
fi

if [[ -n "${KONCLUDE_BIN:-}" ]] && command -v "${KONCLUDE_BIN}" >/dev/null; then
  echo "Konclude found at ${KONCLUDE_BIN} (manual DL baseline — not run in default CI)"
fi

if [[ -n "${HERMIT_JAR:-}" ]] && [[ -f "${HERMIT_JAR}" ]]; then
  echo "HermiT JAR found at ${HERMIT_JAR} (manual cross-check — not run in default CI)"
fi

echo "Tier C harness: smoke checks passed"
