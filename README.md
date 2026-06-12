# OntoLogos

[![CI](https://github.com/eddiethedean/ontologos/actions/workflows/ci.yml/badge.svg)](https://github.com/eddiethedean/ontologos/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/ontologos-core.svg)](https://crates.io/crates/ontologos-core)
[![docs.rs](https://docs.rs/ontologos-core/badge.svg)](https://docs.rs/ontologos-core)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)
![Rust](https://img.shields.io/badge/rust-1.88%2B-orange)

A modular Rust ontology reasoner **in early development**.

**Display name:** OntoLogos · **Crates:** `ontologos-*` · **CLI binary:** `ontologos`

**v0.2 (today):** OWL file parsing and profile detection via [`ontologos-parser`](crates/ontologos-parser) and [`ontologos-profile`](crates/ontologos-profile), plus the in-memory model ([`ontologos-core`](crates/ontologos-core)).

**Planned:** RDFS/RL/EL reasoning (v0.3–v0.5), full CLI workflows, Python bindings (v0.9).

If you need OWL classification today, use Protégé with HermiT or ELK. If you want to embed a Rust ontology graph, load OWL files, or evaluate the architecture, start below.

## What works in v0.2

| Feature | Status |
|---------|--------|
| IRI intern pool, entity registry, axiom store | Available |
| `OntologyBuilder` programmatic construction | Available |
| JSON snapshot v2 (`to_json` / `from_json`) | Available |
| Axiom indexes (subclass, subproperty, equivalence, …) | Available |
| OWL file loading (`.owl`, `.rdf`, `.ttl`, `.ofn`) | Available |
| Profile detection (EL / RL / QL / DL) | Available |
| `ontologos profile` CLI | Available |
| RDFS / RL / EL reasoning | v0.3–v0.5 |
| `classify` / `materialize` / `explain` CLI | v0.3–v0.6 (see [ROADMAP](ROADMAP.md)) |
| Python bindings (full API) | v0.9 |

## Install (library)

Requires **Rust 1.88+**.

From [crates.io](https://crates.io/crates/ontologos-core):

```toml
[dependencies]
ontologos-core = "0.2.0"
ontologos-parser = "0.2.0"
ontologos-profile = "0.2.0"
```

From this repository:

```bash
git clone https://github.com/eddiethedean/ontologos.git
cd ontologos
./benchmarks/scripts/download.sh   # for Pizza corpus examples
```

API reference: [docs.rs/ontologos-core](https://docs.rs/ontologos-core) · [parser](https://docs.rs/ontologos-parser) · [profile](https://docs.rs/ontologos-profile)

> **Python:** `pip install ontologos` is an alpha placeholder only — use Rust crates for v0.2 workflows. See [crates/ontologos-py/README.md](crates/ontologos-py/README.md).

## Quick start (5 minutes)

### Builder + JSON

```bash
cargo run -p ontologos-core --example pizza_builder
```

```rust
use ontologos_core::{Error, Ontology};

fn main() -> Result<(), Error> {
    let ontology = Ontology::builder()
        .class("http://example.org/Pizza")?
        .class("http://example.org/Food")?
        .subclass_of("http://example.org/Pizza", "http://example.org/Food")?
        .build()?;

    let json = ontology.to_json()?;
    let restored = Ontology::from_json(&json)?;
    assert_eq!(restored.axiom_count(), 1);
    Ok(())
}
```

### Load OWL + detect profile (v0.2)

```bash
./benchmarks/scripts/download.sh
cargo build -p ontologos-cli --release
./target/release/ontologos profile benchmarks/data/pizza.owl
```

```rust
use ontologos_parser::load_ontology;
use ontologos_profile::detect_profile;

let ontology = load_ontology(path)?;
let report = detect_profile(&ontology)?;
```

Profile **classification** uses mapped TBox shapes; **diagnostics** may list constructs seen in the source but not stored in core — see [docs/guides/profile-detection.md](docs/guides/profile-detection.md).

Or run: `cargo run -p ontologos-parser --example load_and_profile`

## Workspace

| Crate | Description | Status |
|-------|-------------|--------|
| `ontologos-core` | Core data model, ontology graph, and reasoner API | **v0.2** |
| `ontologos-parser` | OWL/RDF parsers (horned-owl integration) | **v0.2** |
| `ontologos-profile` | OWL profile detection and diagnostics | **v0.2** |
| `ontologos-rdfs` | RDFS reasoning engine | Stub |
| `ontologos-rl` | OWL RL forward-chaining rules | Stub |
| `ontologos-el` | OWL EL classification | Stub |
| `ontologos-query` | Query interface over classified ontologies | Stub |
| `ontologos-explain` | Proof graphs and explanation export | Stub |
| `ontologos-cli` | `ontologos` command-line tool | **Partial** (`profile` only) |
| `ontologos-py` | Python bindings via PyO3 | Stub |

`ontologos-core`, `ontologos-parser`, and `ontologos-profile` are published to crates.io in v0.2.0.

## CLI

```bash
cargo build -p ontologos-cli --release
./target/release/ontologos profile benchmarks/data/pizza.owl
# detected profile: El

./target/release/ontologos --format json profile benchmarks/data/family.owl
```

`classify`, `materialize`, and `explain` load the ontology then return `NotImplemented` until their roadmap milestones (`materialize` v0.3, `classify` v0.5, `explain` v0.6) — see [ROADMAP.md](ROADMAP.md).

## Documentation

| Section | Link |
|---------|------|
| **Getting started** | [docs/README.md](docs/README.md) |
| **Guides** | [Load OWL](docs/getting-started/load-owl-file.md) · [Profile detection](docs/guides/profile-detection.md) · [Security](docs/security.md) |
| **Reference** | [Errors](docs/reference/errors.md) · [CLI](docs/reference/cli.md) · [JSON v2](docs/json-snapshot-v2.md) |
| **Project** | [ROADMAP](ROADMAP.md) · [CHANGELOG](CHANGELOG.md) · [CONTRIBUTING](CONTRIBUTING.md) · [FAQ](FAQ.md) |

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
