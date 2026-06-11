# OntoLogos

[![CI](https://github.com/eddiethedean/ontologos/actions/workflows/ci.yml/badge.svg)](https://github.com/eddiethedean/ontologos/actions/workflows/ci.yml)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)
![Rust](https://img.shields.io/badge/rust-1.78%2B-orange)

A modular Rust ontology reasoner **in early development**.

**Display name:** OntoLogos · **Crates:** `ontologos-*` · **CLI binary:** `ontologos`

**v0.1 (today):** in-memory ontology data model ([`ontologos-core`](crates/ontologos-core)) — build ontologies in Rust or load JSON v2 snapshots.

**Planned:** OWL file parsing (v0.2), profile detection (v0.2), RDFS/RL/EL reasoning (v0.3–v0.5), CLI and Python bindings.

If you need OWL classification today, use Protégé with HermiT or ELK. If you want to embed a Rust ontology graph or evaluate the architecture, start below.

## What works in v0.1

| Feature | Status |
|---------|--------|
| IRI intern pool, entity registry, axiom store | Available |
| `OntologyBuilder` programmatic construction | Available |
| JSON snapshot v2 (`to_json` / `from_json`) | Available |
| Axiom indexes (subclass, subproperty, equivalence, …) | Available |
| OWL file loading (`.owl`, `.ttl`, …) | v0.2 |
| Profile detection | v0.2 |
| RDFS / RL / EL reasoning | v0.3–v0.5 |
| CLI subcommands beyond load | v0.2+ |
| Python bindings | v0.9 |

## Install (library)

Requires **Rust 1.78+**.

From crates.io (after first publish):

```toml
[dependencies]
ontologos-core = "0.1"
```

From this repository:

```bash
git clone https://github.com/eddiethedean/ontologos.git
cd ontologos
```

API reference: [docs.rs/ontologos-core](https://docs.rs/ontologos-core) (after publish)

## Quick start (5 minutes)

```bash
cargo run -p ontologos-core --example pizza_builder
```

Expected output (abbreviated):

```text
Pizza direct superclasses: [EntityId(1)]
Superclass matches Thing: true
Entities: 4
Axioms: 2
Round-trip equal: true
```

### Library usage

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

`Ontology::from_file` returns `Error::ParseNotAvailable` until v0.2 parser support.

## Workspace

| Crate | Description | Status |
|-------|-------------|--------|
| `ontologos-core` | Core data model, ontology graph, and reasoner API | **v0.1** |
| `ontologos-parser` | OWL/RDF parsers (horned-owl integration) | Stub |
| `ontologos-profile` | OWL profile detection and diagnostics | Stub |
| `ontologos-rdfs` | RDFS reasoning engine | Stub |
| `ontologos-rl` | OWL RL forward-chaining rules | Stub |
| `ontologos-el` | OWL EL classification | Stub |
| `ontologos-query` | Query interface over classified ontologies | Stub |
| `ontologos-explain` | Proof graphs and explanation export | Stub |
| `ontologos-cli` | `ontologos` command-line tool | Stub |
| `ontologos-py` | Python bindings via PyO3 | Stub |

Only `ontologos-core` is published to crates.io in v0.1.0.

## CLI (stub until v0.2)

The binary builds, but all subcommands fail at ontology load today:

```bash
cargo build -p ontologos-cli --release
./target/release/ontologos profile test.owl
# error: ontology file parsing is not available until v0.2
```

Planned commands: `profile`, `classify`, `materialize`, `explain` — see [ROADMAP.md](ROADMAP.md).

## Python API (v0.9+)

Python bindings are stubbed; file loading requires v0.2.

## Documentation

- [docs/README.md](docs/README.md) — documentation index and learning path
- [ROADMAP.md](ROADMAP.md) — versioned release plan (0.1 → 1.0 → 2.0)
- [CHANGELOG.md](CHANGELOG.md) — release history
- [CONTRIBUTING.md](CONTRIBUTING.md) — development workflow
- [FAQ.md](FAQ.md) — common questions
- [SPEC.md](SPEC.md) — technical specification (status-tagged)
- [docs/json-snapshot-v2.md](docs/json-snapshot-v2.md) — JSON snapshot format
- [docs/security.md](docs/security.md) — untrusted input guidance
- [docs/comparison.md](docs/comparison.md) — vs HermiT, ELK, and others
- [docs/internal/research/](docs/internal/research/) — maintainer research notes

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
