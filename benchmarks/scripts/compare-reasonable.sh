#!/usr/bin/env bash
# Compare ontologos-rl saturation output against the `reasonable` OWL RL reasoner CLI.
# Optional CI harness — requires `reasonable` on PATH and a built `ontologos` binary.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DATA="${ROOT}/benchmarks/data"
ONTOLOGOS="${ONTOLOGOS_BIN:-${ROOT}/target/release/ontologos}"

if ! command -v reasonable >/dev/null 2>&1; then
  echo "reasonable CLI not found on PATH; install from https://github.com/gtfierro/reasonable" >&2
  exit 1
fi

if [[ ! -x "${ONTOLOGOS}" ]]; then
  echo "ontologos binary not found at ${ONTOLOGOS}; run: cargo build -p ontologos-cli --release" >&2
  exit 1
fi

compare_corpus() {
  local name="$1"
  local file="$2"
  echo "==> ${name}"
  local tmp
  tmp="$(mktemp -d)"
  trap 'rm -rf "${tmp}"' RETURN

  # Export normalized axiom counts via JSON materialize (RL path when profile detects RL).
  "${ONTOLOGOS}" --format json materialize "${file}" >"${tmp}/ontologos.json"
  reasonable materialize "${file}" >"${tmp}/reasonable.ttl" 2>/dev/null || {
    echo "reasonable materialize failed for ${file}" >&2
    return 1
  }

  local onto_count reasonable_lines
  onto_count="$(jq '.final_axiom_count // .axiom_count // empty' "${tmp}/ontologos.json")"
  reasonable_lines="$(wc -l <"${tmp}/reasonable.ttl" | tr -d ' ')"

  echo "  ontologos final_axiom_count: ${onto_count:-unknown}"
  echo "  reasonable materialized lines: ${reasonable_lines}"
  echo "  (full triple diff not automated in v0.4 — inspect ${tmp} manually)"
}

compare_corpus "family" "${DATA}/family.owl"
if [[ -f "${DATA}/brick-subset.ttl" ]]; then
  compare_corpus "brick-subset" "${DATA}/brick-subset.ttl"
fi

echo "Done."
