# CLI installation


--8<-- "snippets/before-integrate-callout.md"
The `ontologos` command-line tool is **not published to crates.io**. Install from git or build from a clone.

Install channels: [Install and channels](../guides/install-channels.md). Command reference: [CLI](../reference/cli.md).

## Prerequisites

- Rust **1.88+** (`rustc --version` must be >= 1.88.0)
- No Java or Protégé required

```bash
rustup update stable
rustc --version
```

If `rustc` is too old, see [Troubleshooting — rustc version](../guides/troubleshooting.md#rustc-version-too-old).

## Option 1 — Install from git (recommended)

Tagged release (matches PyPI/crates.io **1.0.0**):

```bash
cargo install --git https://github.com/eddiethedean/ontologos --tag v1.1.4 ontologos-cli
ontologos --help
```

Latest `main` (development):

```bash
cargo install --git https://github.com/eddiethedean/ontologos ontologos-cli
```

The binary is installed to `~/.cargo/bin/ontologos`. Ensure that directory is on your `PATH`.

## Option 2 — Build from a clone

```bash
git clone https://github.com/eddiethedean/ontologos.git
cd ontologos
cargo build -p ontologos-cli --release
./target/release/ontologos --help
```

Optional: add a symlink or alias to `target/release/ontologos`.

## Verify installation

Download the vendored Family corpus (no clone required):

```bash
curl -L -o family.owl \
  https://raw.githubusercontent.com/eddiethedean/ontologos/main/benchmarks/data/family.owl
ontologos profile family.owl
ontologos classify --profile rl family.owl
```

**Expected:** `detected: RL` for Family; classify reports inferred axioms > 0.

## Common commands

```bash
ontologos profile ontology.owl              # detect OWL profile
ontologos materialize ontology.owl          # RDFS TBox materialization
ontologos classify --profile auto file.owl  # routed classification
ontologos classify --profile dl --budget-secs 30 file.owl  # OWL DL
ontologos explain --profile el pizza.owl    # proof graph (EL)
```

## Benchmark corpora (clone only)

`family.owl` works without a clone. Pizza and HermiT fixtures require:

```bash
./benchmarks/scripts/download.sh
```

See [Benchmarks](../project/benchmarks.md).

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| `command not found: ontologos` | Add `~/.cargo/bin` to `PATH`, or use full path to binary |
| `error: package(s) ontologos-cli not found` on crates.io | CLI is git-only — use `cargo install --git ...` above |
| File not found for Pizza | Fetch with `curl -L -o pizza.owl https://github.com/owlcs/pizza-ontology/raw/refs/heads/master/pizza.owl` or run `download.sh` from a clone |

Full list: [Troubleshooting](../guides/troubleshooting.md).

## Related

- [CLI reference](../reference/cli.md)
- [Evaluator playbook](../guides/evaluator-playbook.md)
- [Python guide](../guides/python.md) — no Rust required
