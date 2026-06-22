# OntoLogos

[![CI](https://github.com/eddiethedean/ontologos/actions/workflows/ci.yml/badge.svg)](https://github.com/eddiethedean/ontologos/actions/workflows/ci.yml)
[![Documentation](https://readthedocs.org/projects/ontologos/badge/?version=latest)](https://ontologos.readthedocs.io/en/latest/)
[![crates.io](https://img.shields.io/crates/v/ontologos-core.svg)](https://crates.io/crates/ontologos-core)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)
![Rust](https://img.shields.io/badge/rust-1.88%2B-orange)

Native Rust ontology reasoning orchestration: load OWL files, detect profiles, and delegate to
**reasonable** (RL/RDFS) and **horned-owl** (parsing) through stable facades. **OWL EL** uses the in-house completion engine in `ontologos-el`.

**Try in 5 minutes:** `pip install ontologos` or add `ontologos-parser = "1.0.0"` to Cargo.toml — no clone required.

**Status:** **v1.0.0** workspace on `main` — HermiT parity in progress (**593** active conformance tests). See [Release status](docs/project/release-status.md) and [HermiT parity gap report](docs/internal/hermit-parity-gap-report.md).

| You need… | Use today |
|-----------|-----------|
| Embed ontology graph in Rust | `ontologos-core` |
| Load `.owl` / `.ttl` files | `ontologos-parser` (horned-owl) |
| RDFS TBox inferences | `ontologos-rdfs` → reasonable |
| OWL RL saturation | `ontologos-rl` → reasonable |
| OWL EL taxonomy | `ontologos-el` (in-house completion) |
| Multi-profile routing | `ontologos-facade` (CLI, Python, DL preview) |
| Engine adapters / conversions | `ontologos-bridge` |

> **New to the workspace?** Start with `pip install ontologos` or `ontologos-parser` + `ontologos-rdfs`. See [Choosing an API](docs/guides/choosing-an-api.md).

**5-minute try:** [Getting started](https://ontologos.readthedocs.io/en/latest/getting-started/) · **API:** [docs.rs/ontologos-core](https://docs.rs/ontologos-core/1.0.0)

> **Partial OWL mapping:** `axiom_count()` reflects mapped axioms, not Protégé's total. See [Supported constructs](docs/reference/supported-constructs.md).

## Install

Requires **Rust 1.88+**.

```toml
[dependencies]
ontologos-core = "1.0.0"
ontologos-parser = "1.0.0"
ontologos-profile = "1.0.0"
ontologos-bridge = "1.0.0"
ontologos-rdfs = "1.0.0"
ontologos-rl = "1.0.0"
ontologos-el = "1.0.0"
ontologos-query = "1.0.0"
ontologos-explain = "1.0.0"
```

**Python:** `pip install ontologos` — `Ontology`, `Reasoner`, `explain()`, incremental mutations ([Python guide](docs/guides/python.md)).

## Quick start (crates.io)

No repository clone required — download a sample ontology:

```bash
curl -L -o family.owl \
  https://raw.githubusercontent.com/eddiethedean/ontologos/main/benchmarks/data/family.owl
cargo new ontologos-demo && cd ontologos-demo
```

`Cargo.toml`:

```toml
[package]
name = "ontologos-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
ontologos-core = "1.0.0"
ontologos-parser = "1.0.0"
ontologos-rdfs = "1.0.0"
```

`src/main.rs`:

```rust
use ontologos_parser::load_ontology;
use ontologos_rdfs::RdfsEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("family.owl");
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
| `ontologos-watch` | File-watch reload (Ontocode hook) | Workspace only |
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
| **v1.0.0 release** | [Release notes](.github/release/v1.0.0.md) · [Migration guide](docs/migration/v0.8.x-to-v1.0.0.md) |

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
