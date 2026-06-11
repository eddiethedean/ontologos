#!/usr/bin/env bash
set -euo pipefail

# Publish workspace crates in dependency order. crates.io index propagation
# can lag behind uploads, so each crate is retried before failing the job.
#
# v0.2.0: core, parser, and profile crates publish to crates.io.

CRATES=(
  ontologos-core
  ontologos-parser
  ontologos-profile
  # ontologos-rdfs
  # ontologos-rl
  # ontologos-el
  # ontologos-query
  # ontologos-explain
  # ontologos-cli
  # ontologos-py
)

publish_crate() {
  local crate="$1"
  local attempt

  for attempt in 1 2 3 4 5 6; do
    echo "Publishing ${crate} (attempt ${attempt}/6)..."
    if cargo publish -p "${crate}" --locked; then
      echo "Published ${crate}"
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
