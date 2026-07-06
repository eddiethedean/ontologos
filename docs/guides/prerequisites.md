# Prerequisites

OntoLogos is a **Rust workspace** with optional **Python bindings** and **source-build native bindings** (v1.1.1 on `main`). Pick the row that matches your task. Install channels: [Install and channels](install-channels.md).

Unfamiliar with OWL terms? See the [Glossary](glossary.md).

## Decision table

| Task | Rust | Python | Other toolchain | Clone repo? |
|------|------|--------|-----------------|-------------|
| Embed reasoning in a Rust app (crates.io) | **1.88+** | — | — | No |
| `pip install ontologos` | — | **3.10+** (wheels on Linux/macOS/Windows) | — | No |
| Build `ontologos-cli` from source | **1.88+** | — | — | Yes |
| Build **Node.js** bindings | **1.88+** | — | **Node.js 18+**, npm | Yes |
| Build **Java** bindings | **1.88+** | — | **JDK 17+**, **Maven 3.9+** | Yes |
| Build **.NET** bindings | **1.88+** | — | **.NET 8+** SDK | Yes |
| Build **C/C++** bindings | **1.88+** | — | C/C++ compiler, CMake (optional) | Yes |
| Build **WASM** bindings | **1.88+** + `wasm32-unknown-unknown` | — | **wasm-pack**, Node.js | Yes |
| Run full conformance / benchmarks | **1.88+** | 3.10+ optional | JDK 17+ for Tier C (optional) | Yes + `./benchmarks/scripts/download.sh` |
| MSRV CI gate | **1.88** exactly | — | — | Yes |
| Contribute (fmt, clippy, tests) | **stable** (1.88+) | 3.10+ for `ontologos-py` | Per binding row above | Yes |

**MSRV:** Rust **1.88** (workspace `Cargo.toml` `rust-version`; CI `msrv` job enforces exactly 1.88).

## Rust toolchain

```bash
rustup update stable
rustup component add rustfmt clippy
rustc --version   # must be >= 1.88.0
```

If `rustc` is too old, see [Troubleshooting — rustc version](troubleshooting.md#rustc-version-too-old).

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

Tagged install: [CLI installation](../getting-started/cli-install.md) (`--tag v1.1.1`).

## Native bindings (source-build)

See [Bindings overview](bindings-overview.md) for end-to-end build steps per language.

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
| npm/Maven/NuGet packages | Bindings are source-build until v1.1.1 publishes |

## Next step

[Start here](start-here.md) — pick a persona path.
