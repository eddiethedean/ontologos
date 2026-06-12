# OntoLogos

[![CI](https://github.com/eddiethedean/ontologos/actions/workflows/ci.yml/badge.svg)](https://github.com/eddiethedean/ontologos/actions/workflows/ci.yml)
[![Documentation](https://readthedocs.org/projects/ontologos/badge/?version=latest)](https://ontologos.readthedocs.io/en/latest/)
[![crates.io](https://img.shields.io/crates/v/ontologos-core.svg)](https://crates.io/crates/ontologos-core)
[![docs.rs](https://docs.rs/ontologos-core/badge.svg)](https://docs.rs/ontologos-core/0.4.0/ontologos_core/)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)
![Rust](https://img.shields.io/badge/rust-1.88%2B-orange)

A modular Rust ontology reasoner **in early development** — embeddable OWL graph, file loading, profile detection, RDFS materialization, and OWL RL saturation.

**Display name:** OntoLogos · **Crates:** `ontologos-*` · **CLI binary:** `ontologos`

Replace JVM-bound reasoning in Rust and Python pipelines with a native, maintained multi-profile stack. **Not a HermiT/ELK replacement yet** — use Protégé + ELK for production EL/DL classification today.

| Layer | Status |
|-------|--------|
| Core model + parser + profiles | Available (v0.4) |
| RDFS + OWL RL engines | Available (library + Python alpha) |
| OWL EL taxonomy classification | v0.5 |
| Explanations + full Python API | v0.6 / v0.9 |

> **Partial OWL mapping:** OntoLogos maps a **subset** of OWL axioms into its core model. `axiom_count()` reflects mapped axioms, not Protégé's total. See [Supported constructs](docs/reference/supported-constructs.md) before comparing results.

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
| `classify` CLI (RDFS only — not OWL taxonomy classification) | Available |
| OWL EL reasoning | v0.5 |
| `explain` CLI | Stub (v0.6 — see [ROADMAP](ROADMAP.md)) |
| Python bindings (`profile=rdfs` / `rl`) | Alpha (full API v0.9) |

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
./benchmarks/scripts/download.sh   # required for Pizza examples and full test suite
```

> **Repository clone:** Run `./benchmarks/scripts/download.sh` before `cargo test --workspace`. The Family corpus is vendored; Pizza and other benchmarks are downloaded. See [benchmarks/README.md](benchmarks/README.md).

API reference: [docs.rs/ontologos-core](https://docs.rs/ontologos-core/0.4.0) · [parser](https://docs.rs/ontologos-parser/0.4.0) · [profile](https://docs.rs/ontologos-profile/0.4.0) · [rdfs](https://docs.rs/ontologos-rdfs/0.4.0) · [rl](https://docs.rs/ontologos-rl/0.4.0) · User guide: [ontologos.readthedocs.io](https://ontologos.readthedocs.io/en/latest/)

> **Python:** `pip install ontologos` is an alpha package — `Reasoner(path, profile="rdfs").classify()` runs RDFS materialization; `profile="rl"` runs OWL RL saturation. The default profile is not implemented until v0.5. Full Python APIs ship in v0.9. See [docs/guides/python.md](docs/guides/python.md).

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

### Load OWL + RDFS materialize

```bash
cargo build -p ontologos-cli --release
./target/release/ontologos materialize benchmarks/data/family.owl
./target/release/ontologos profile benchmarks/data/pizza.owl   # after download.sh
```

```rust
use ontologos_parser::load_ontology;
use ontologos_rdfs::RdfsEngine;

let mut ontology = load_ontology(path)?;
let report = RdfsEngine::new().materialize(&mut ontology)?;
```

### Load OWL + OWL RL saturation (v0.4)

```rust
use ontologos_parser::load_ontology;
use ontologos_rl::RlEngine;

let mut ontology = load_ontology(path)?;
let report = RlEngine::new(1).saturate(&mut ontology)?;
println!(
    "inferred {} axioms ({} from RDFS)",
    report.inferred_total(),
    report.rdfs_inferred
);
```

Or run: `cargo run -p ontologos-rl --example rl_saturation`

> **CLI note:** `ontologos classify` runs **RDFS materialization only** (same inferences as `materialize`). For OWL RL, use the library above or Python `profile="rl"`. OWL EL taxonomy classification arrives in v0.5.

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

The CLI is **source-build only** (`ontologos-cli` is not published to crates.io). Build from a repository clone:

```bash
cargo build -p ontologos-cli --release
./target/release/ontologos profile benchmarks/data/pizza.owl
./target/release/ontologos materialize benchmarks/data/family.owl
./target/release/ontologos classify benchmarks/data/family.owl
```

Or install from git (requires Rust 1.88+):

```bash
cargo install --git https://github.com/eddiethedean/ontologos ontologos-cli
```

> **`classify` is not OWL taxonomy classification.** It runs RDFS TBox materialization (same inferences as `materialize`; only the `status` field differs). OWL RL saturation is available via `ontologos-rl` (library) or Python `profile="rl"`. OWL EL taxonomy classification and CLI profile routing ship in v0.5. `explain` is a stub until v0.6 — see [CLI reference](docs/reference/cli.md).

## Documentation

| Section | Link |
|---------|------|
| **Documentation site** | [ontologos.readthedocs.io](https://ontologos.readthedocs.io/) |
| **Getting started** | [docs/index.md](docs/index.md) |
| **Guides** | [Load OWL](docs/getting-started/load-owl-file.md) · [OWL RL](docs/getting-started/owl-rl-saturation.md) · [Profile detection](docs/guides/profile-detection.md) · [Choosing an API](docs/guides/choosing-an-api.md) · [Python](docs/guides/python.md) · [Security](docs/security.md) |
| **Reference** | [Architecture](docs/architecture.md) · [Errors](docs/reference/errors.md) · [CLI](docs/reference/cli.md) · [JSON v2](docs/json-snapshot-v2.md) · [RL rules](docs/reference/rl-rules.md) · [Conformance](docs/reference/conformance.md) |
| **Project** | [ROADMAP](ROADMAP.md) · [CHANGELOG](CHANGELOG.md) · [CONTRIBUTING](CONTRIBUTING.md) · [FAQ](FAQ.md) · [Docs build](docs/readthedocs.md) |

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
