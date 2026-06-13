# OntoLogos

[![CI](https://github.com/eddiethedean/ontologos/actions/workflows/ci.yml/badge.svg)](https://github.com/eddiethedean/ontologos/actions/workflows/ci.yml)
[![Documentation](https://readthedocs.org/projects/ontologos/badge/?version=latest)](https://ontologos.readthedocs.io/en/latest/)
[![crates.io](https://img.shields.io/crates/v/ontologos-core.svg)](https://crates.io/crates/ontologos-core)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)
![Rust](https://img.shields.io/badge/rust-1.88%2B-orange)

Native Rust ontology reasoning orchestration: load OWL files, detect profiles, and delegate to
**reasonable** (RL/RDFS) and **horned-owl** (parsing) through stable facades. **OWL EL** uses the in-house completion engine in `ontologos-el`.

**Status:** **v0.8.0** on `main` — incremental EL/RL/RDFS via `ReasonerConfig::incremental`. **Not** full OWL DL / HermiT replacement.

| You need… | Use today |
|-----------|-----------|
| Embed ontology graph in Rust | `ontologos-core` |
| Load `.owl` / `.ttl` files | `ontologos-parser` (horned-owl) |
| RDFS TBox inferences | `ontologos-rdfs` → reasonable |
| OWL RL saturation | `ontologos-rl` → reasonable |
| OWL EL taxonomy | `ontologos-el` (in-house completion) |
| Engine adapters / conversions | `ontologos-bridge` |

**5-minute try:** [Getting started](https://ontologos.readthedocs.io/en/latest/getting-started/) · **API:** [docs.rs/ontologos-core](https://docs.rs/ontologos-core/0.7.0)

> **Partial OWL mapping:** `axiom_count()` reflects mapped axioms, not Protégé's total. See [Supported constructs](docs/reference/supported-constructs.md).

## Install

Requires **Rust 1.88+**.

```toml
[dependencies]
ontologos-core = "0.8.0"
ontologos-parser = "0.8.0"
ontologos-profile = "0.8.0"
ontologos-bridge = "0.8.0"
ontologos-rdfs = "0.8.0"
ontologos-rl = "0.8.0"
ontologos-el = "0.8.0"
ontologos-query = "0.8.0"
ontologos-explain = "0.8.0"
```

**Python (alpha):** `pip install ontologos` — pass `profile="rdfs"`, `"rl"`, `"el"`, or `"auto"` ([Python guide](docs/guides/python.md)).

## Quick start (crates.io)

No repository clone required — use any OWL file on disk:

```bash
cargo new ontologos-demo && cd ontologos-demo
```

`Cargo.toml`:

```toml
[package]
name = "ontologos-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
ontologos-core = "0.7.0"
ontologos-parser = "0.7.0"
ontologos-rdfs = "0.7.0"
```

`src/main.rs`:

```rust
use ontologos_parser::load_ontology;
use ontologos_rdfs::RdfsEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("ontology.owl"); // your file here
    let mut ontology = load_ontology(path)?;

    let report = RdfsEngine::new().materialize(&mut ontology)?;
    println!(
        "mapped {} → {} axioms (inferred {})",
        report.initial_axiom_count,
        report.final_axiom_count,
        report.inferred_total()
    );
    Ok(())
}
```

```bash
cargo run
```

OWL RL saturation, profile detection, and CLI examples: [documentation site](https://ontologos.readthedocs.io/en/latest/getting-started/).

## Quick start (repository clone)

For CLI, benchmarks, and full test suite:

```bash
git clone https://github.com/eddiethedean/ontologos.git
cd ontologos
./benchmarks/scripts/download.sh   # Pizza corpus; Family is vendored
cargo run -p ontologos-core --example pizza_builder
cargo build -p ontologos-cli --release
./target/release/ontologos materialize benchmarks/data/family.owl
```

> **CLI:** `classify --profile auto` routes to EL or RL; use `materialize` for explicit RDFS. See [CLI reference](docs/reference/cli.md).

## Workspace

| Crate | Description | Published |
|-------|-------------|-----------|
| `ontologos-core` | Data model, builder, JSON v2 | crates.io |
| `ontologos-bridge` | core ↔ horned-owl/oxrdf adapters | crates.io |
| `ontologos-parser` | OWL/RDF loading (horned-owl) | crates.io |
| `ontologos-profile` | Profile detection | crates.io |
| `ontologos-rdfs` | RDFS facade → reasonable | crates.io |
| `ontologos-rl` | OWL RL facade → reasonable | crates.io |
| `ontologos-el` | OWL EL completion engine | crates.io |
| `ontologos-explain` | Proof graphs and explanations | crates.io |
| `ontologos-query` | Taxonomy queries | crates.io |
| `ontologos-cli` | CLI binary | Source-build only |
| `ontologos-py` | Python bindings | PyPI (alpha) |

Full feature matrix and roadmap: [documentation index](docs/index.md) · [ROADMAP](ROADMAP.md)

## Documentation

| Section | Link |
|---------|------|
| **Documentation site** | [ontologos.readthedocs.io](https://ontologos.readthedocs.io/) |
| **Getting started** | [docs/getting-started/](docs/getting-started/index.md) |
| **Guides** | [RDFS](docs/getting-started/rdfs-materialization.md) · [OWL RL](docs/getting-started/owl-rl-saturation.md) · [Python](docs/guides/python.md) · [Comparison](docs/comparison.md) |
| **Reference** | [CLI](docs/reference/cli.md) · [Errors](docs/reference/errors.md) · [Supported constructs](docs/reference/supported-constructs.md) |
| **Project** | [FAQ](FAQ.md) · [CONTRIBUTING](CONTRIBUTING.md) · [CHANGELOG](CHANGELOG.md) |

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
