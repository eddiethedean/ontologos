#!/usr/bin/env bash
# Compare ontologos-rl saturation against the `reasonable` engine (library path).
# CI gate: runs the Rust harness that diffs triple closures on mapped core axioms.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "${ROOT}"

echo "==> ontologos-rl vs reasonable (family.owl triple closure)"
cargo test -p ontologos-rl family_rl_closure_matches_reasonable --locked -- --nocapture

if command -v reasonable >/dev/null 2>&1; then
  ONTOLOGOS="${ONTOLOGOS_BIN:-${ROOT}/target/release/ontologos}"
  DATA="${ROOT}/benchmarks/data"
  if [[ -x "${ONTOLOGOS}" && -f "${DATA}/family.owl" ]]; then
    echo "==> optional CLI smoke (family.owl axiom counts)"
    tmp="$(mktemp -d)"
    trap 'rm -rf "${tmp}"' RETURN
    "${ONTOLOGOS}" --format json materialize "${DATA}/family.owl" >"${tmp}/ontologos.json"
    reasonable materialize "${DATA}/family.owl" >"${tmp}/reasonable.ttl" 2>/dev/null || true
    if command -v jq >/dev/null 2>&1; then
      onto_count="$(jq '.final_axiom_count // empty' "${tmp}/ontologos.json")"
      echo "  ontologos final_axiom_count: ${onto_count:-unknown}"
    fi
    if [[ -f "${tmp}/reasonable.ttl" ]]; then
      echo "  reasonable materialized lines: $(wc -l <"${tmp}/reasonable.ttl" | tr -d ' ')"
    fi
  fi
else
  echo "reasonable CLI not on PATH (optional); library harness passed."
fi

echo "Done."
