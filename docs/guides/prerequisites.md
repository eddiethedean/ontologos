# Prerequisites

OntoLogos is a **Rust workspace** with optional **Python bindings**. Pick the row that matches your task.

## Decision table

| Task | Rust | Python | Clone repo? |
|------|------|--------|-------------|
| Embed reasoning in a Rust app (crates.io) | **1.88+** | — | No |
| `pip install ontologos` | — | **3.10+** (wheels on Linux/macOS/Windows) | No |
| Build `ontologos-cli` from source | **1.88+** | — | Yes |
| Run full conformance / benchmarks | **1.88+** | 3.10+ optional | Yes + `./benchmarks/scripts/download.sh` |
| MSRV CI gate | **1.88** exactly | — | Yes |
| Contribute (fmt, clippy, tests) | **stable** (1.88+) | 3.10+ for `ontologos-py` | Yes |

**MSRV:** Rust **1.88** (see root `Cargo.toml`). CI enforces MSRV on every push.

## Rust toolchain

```bash
rustup update stable
rustup component add rustfmt clippy
rustc --version   # must be >= 1.88.0
```

New projects from crates.io only need a standard Cargo workspace—no fork of this repository.

## Python toolchain

```bash
pip install ontologos
python -c "from ontologos import Reasoner; print('ok')"
```

Optional extras: `pip install 'ontologos[pandas]'` or `'ontologos[polars]'` for DataFrame export.

Development install from a clone: see [Python guide](python.md#install).

## CLI from git (not on crates.io)

```bash
git clone https://github.com/eddiethedean/ontologos.git
cd ontologos
cargo build -p ontologos-cli --release
./target/release/ontologos --help
```

## Benchmark corpora (clone only)

Family ontology is vendored. Pizza and other HermiT fixtures:

```bash
./benchmarks/scripts/download.sh
```

## What you do **not** need

| Misconception | Reality |
|---------------|---------|
| Protégé installed | Optional for authoring OWL; not required to run OntoLogos |
| Java / HermiT | OntoLogos does not embed HermiT; conformance tests compare against ported fixtures |
| Every crate on crates.io | `ontologos-cli` and `ontologos-conformance` are source-build only |

## Next step

[Start here](start-here.md) — pick a persona path.
