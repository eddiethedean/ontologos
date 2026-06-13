#!/usr/bin/env bash
set -euo pipefail

# Publish workspace crates in dependency order. crates.io index propagation
# can lag behind uploads, so each crate is retried before failing the job.
#
# v0.6.x: core → bridge → profile → parser → rdfs → rl → el → query → explain.

CRATES=(
  ontologos-core
  ontologos-bridge
  ontologos-profile
  ontologos-parser
  ontologos-rdfs
  ontologos-rl
  ontologos-el
  ontologos-query
  ontologos-explain
  # ontologos-cli
  # ontologos-py
)

already_published() {
  local output="$1"
  grep -qE 'already exists on crates.io|is already uploaded' <<<"${output}"
}

publish_crate() {
  local crate="$1"
  local attempt
  local output

  for attempt in 1 2 3 4 5 6; do
    echo "Publishing ${crate} (attempt ${attempt}/6)..."
    if output="$(cargo publish -p "${crate}" --locked 2>&1)"; then
      echo "${output}"
      echo "Published ${crate}"
      return 0
    fi
    echo "${output}"

    if already_published "${output}"; then
      echo "${crate} already on crates.io; skipping"
      return 0
    fi

    if [ "${attempt}" -eq 6 ]; then
      echo "Failed to publish ${crate}"
      return 1
    fi

    echo "Waiting for crates.io index to update..."
    sleep 30
  done
}

for crate in "${CRATES[@]}"; do
  publish_crate "${crate}"
done
