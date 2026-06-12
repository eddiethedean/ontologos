# OntoLogos

[![CI](https://github.com/eddiethedean/ontologos/actions/workflows/ci.yml/badge.svg)](https://github.com/eddiethedean/ontologos/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/ontologos-core.svg)](https://crates.io/crates/ontologos-core)
[![docs.rs](https://docs.rs/ontologos-core/badge.svg)](https://docs.rs/ontologos-core)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)
![Rust](https://img.shields.io/badge/rust-1.88%2B-orange)

A modular Rust ontology reasoner **in early development**.

**Display name:** OntoLogos · **Crates:** `ontologos-*` · **CLI binary:** `ontologos`

**v0.4 (today):** OWL RL forward-chaining via [`ontologos-rl`](crates/ontologos-rl) (on top of RDFS), full ABox in core, plus parsing and profile detection from v0.2–v0.3.

**Planned:** OWL EL reasoning (v0.5), explanations (v0.6), Python bindings (v0.9).

If you need OWL classification today, use Protégé with HermiT or ELK. If you want to embed a Rust ontology graph, load OWL files, or evaluate the architecture, start below.

## What works in v0.4

| Feature | Status |
|---------|--------|
| IRI intern pool, entity registry, axiom store | Available |
| `OntologyBuilder` programmatic construction | Available |
| JSON snapshot v2 (`to_json` / `from_json`) | Available |
| ABox axioms (class/property assertions, same/different) | Available |
| Axiom indexes (subclass, subproperty, equivalence, ABox, …) | Available |
| OWL file loading (`.owl`, `.rdf`, `.ttl`, `.ofn`) | Available |
| Profile detection (EL / RL / QL / DL) | Available |
| RDFS materialization (TBox rules) | Available |
| OWL RL saturation (`RlEngine::saturate`) | Available |
| `ontologos profile` CLI | Available |
| `ontologos materialize` CLI | Available |
| `classify` CLI (RDFS) | Available |
| OWL EL reasoning | v0.5 |
| `explain` CLI | v0.6 (see [ROADMAP](ROADMAP.md)) |
| Python bindings (`profile=rdfs` / `rl`) | Alpha |

## Install (library)

Requires **Rust 1.88+**.

From [crates.io](https://crates.io/crates/ontologos-core):

```toml
[dependencies]
ontologos-core = "0.4.0"
ontologos-parser = "0.4.0"
ontologos-profile = "0.4.0"
ontologos-rdfs = "0.4.0"
ontologos-rl = "0.4.0"
```

From this repository:

```bash
git clone https://github.com/eddiethedean/ontologos.git
cd ontologos
./benchmarks/scripts/download.sh   # for Pizza corpus examples
```

API reference: [docs.rs/ontologos-core](https://docs.rs/ontologos-core) · [parser](https://docs.rs/ontologos-parser) · [profile](https://docs.rs/ontologos-profile) · [rdfs](https://docs.rs/ontologos-rdfs) · [rl](https://docs.rs/ontologos-rl)

> **Python:** `pip install ontologos` is an alpha package — `Reasoner(path, profile="rdfs").classify()` runs RDFS materialization; `profile="rl"` runs OWL RL saturation. The default profile is not implemented until v0.5. Full Python APIs ship in v0.9. See [crates/ontologos-py/README.md](crates/ontologos-py/README.md).

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

### Load OWL + materialize

```bash
./benchmarks/scripts/download.sh
cargo build -p ontologos-cli --release
./target/release/ontologos materialize benchmarks/data/family.owl
./target/release/ontologos profile benchmarks/data/pizza.owl
```

```rust
use ontologos_parser::load_ontology;
use ontologos_rdfs::RdfsEngine;

let mut ontology = load_ontology(path)?;
let report = RdfsEngine::new().materialize(&mut ontology)?;
```

### Load OWL + detect profile

Profile **classification** uses mapped TBox shapes; **diagnostics** may list constructs seen in the source but not stored in core — see [docs/guides/profile-detection.md](docs/guides/profile-detection.md).

Or run: `cargo run -p ontologos-parser --example load_and_profile`

## Workspace

| Crate | Description | Status |
|-------|-------------|--------|
| `ontologos-core` | Core data model, ontology graph, and reasoner API | **v0.4** |
| `ontologos-parser` | OWL/RDF parsers (horned-owl integration) | **v0.4** |
| `ontologos-profile` | OWL profile detection and diagnostics | **v0.4** |
| `ontologos-rdfs` | RDFS reasoning engine | **v0.4** |
| `ontologos-rl` | OWL RL forward-chaining rules | **v0.4** |
| `ontologos-el` | OWL EL classification | Stub (v0.5) |
| `ontologos-query` | Query interface over classified ontologies | Stub (v0.5) |
| `ontologos-explain` | Proof graphs and explanation export | Stub (v0.6) |
| `ontologos-cli` | `ontologos` command-line tool | **Partial** (`profile`, `materialize`, `classify` RDFS) |
| `ontologos-conformance` | HermiT-ported conformance tests (workspace-only) | Dev harness |
| `ontologos-py` | Python bindings via PyO3 | Alpha (`profile=rdfs` / `rl`) |

**v0.4.0** publishes [`ontologos-core`](https://crates.io/crates/ontologos-core), [`ontologos-parser`](https://crates.io/crates/ontologos-parser), [`ontologos-profile`](https://crates.io/crates/ontologos-profile), [`ontologos-rdfs`](https://crates.io/crates/ontologos-rdfs), and [`ontologos-rl`](https://crates.io/crates/ontologos-rl) to crates.io.

## CLI

```bash
cargo build -p ontologos-cli --release
./target/release/ontologos profile benchmarks/data/pizza.owl
./target/release/ontologos materialize benchmarks/data/family.owl
./target/release/ontologos classify benchmarks/data/family.owl
```

`classify` and `materialize` run RDFS materialization only; OWL RL saturation is available via `ontologos-rl` (library) or Python `profile="rl"`. OWL EL taxonomy classification and CLI profile routing ship in v0.5; `explain` in v0.6 — see [ROADMAP.md](ROADMAP.md).

## Documentation

| Section | Link |
|---------|------|
| **Getting started** | [docs/README.md](docs/README.md) |
| **Guides** | [Load OWL](docs/getting-started/load-owl-file.md) · [Profile detection](docs/guides/profile-detection.md) · [Security](docs/security.md) |
| **Reference** | [Errors](docs/reference/errors.md) · [CLI](docs/reference/cli.md) · [JSON v2](docs/json-snapshot-v2.md) |
| **Project** | [ROADMAP](ROADMAP.md) · [CHANGELOG](CHANGELOG.md) · [CONTRIBUTING](CONTRIBUTING.md) · [FAQ](FAQ.md) |

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
