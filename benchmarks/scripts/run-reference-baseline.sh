#!/usr/bin/env bash
# Run external reference reasoners when KONCLUDE_BIN / HERMIT_JAR are available.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DATA="${ROOT}/benchmarks/data"
ONTO="${DATA}/pizza.owl"

if [[ ! -f "${ONTO}" ]]; then
  echo "skip: ${ONTO} not found" >&2
  exit 0
fi

if [[ -n "${KONCLUDE_BIN:-}" ]] && command -v "${KONCLUDE_BIN}" >/dev/null 2>&1; then
  echo "Konclude reference: ${KONCLUDE_BIN}"
  OUT="$(mktemp)"
  "${KONCLUDE_BIN}" --status "${ONTO}" >"${OUT}" 2>&1 || true
  head -5 "${OUT}" || true
  rm -f "${OUT}"
fi

if [[ -n "${HERMIT_JAR:-}" ]] && [[ -f "${HERMIT_JAR}" ]] && command -v java >/dev/null 2>&1; then
  echo "HermiT reference: ${HERMIT_JAR}"
  java -jar "${HERMIT_JAR}" --classify "${ONTO}" 2>/dev/null | head -5 || true
fi

echo "reference baseline runner: done (optional tools only)"
