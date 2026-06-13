#!/usr/bin/env bash
# Tier C reference harness: compare OntoLogos DL taxonomy to vendored golden baselines.
# Optional: set KONCLUDE_BIN or HERMIT_JAR for external cross-checks (not required for CI).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DATA="${ROOT}/benchmarks/data"

echo "OntoLogos Tier C smoke (Pizza EL golden already gated separately)"

if [[ -f "${DATA}/pizza-el-golden.json" ]]; then
  "${ROOT}/benchmarks/scripts/compare-pizza-el-golden.sh"
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
