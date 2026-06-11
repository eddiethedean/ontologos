# Ontologos

A modular Rust ontology reasoner supporting OWL EL, OWL RL, RDFS reasoning, explanation generation, and incremental classification.

## Workspace

| Crate | Description |
|-------|-------------|
| `ontologos-core` | Core data model, ontology graph, and reasoner API |
| `ontologos-parser` | OWL/RDF parsers (horned-owl integration) |
| `ontologos-profile` | OWL profile detection and diagnostics |
| `ontologos-rdfs` | RDFS reasoning engine |
| `ontologos-rl` | OWL RL forward-chaining rules |
| `ontologos-el` | OWL EL classification |
| `ontologos-query` | Query interface over classified ontologies |
| `ontologos-explain` | Proof graphs and explanation export |
| `ontologos-cli` | `ontologos` command-line tool |
| `ontologos-py` | Python bindings via PyO3 |

## Status (v0.1)

**v0.1** delivers the in-memory core data model (`ontologos-core`). OWL file loading, profile detection, and reasoning land in **v0.2+**. See [ROADMAP.md](ROADMAP.md).

## Quick start

```bash
cargo build
cargo test -p ontologos-core
```

## Rust API (v0.1)

Build an ontology programmatically or load from JSON:

```rust
use ontologos_core::Ontology;

let ontology = Ontology::builder()
    .class("http://example.org/Pizza")?
    .class("http://example.org/Food")?
    .subclass_of("http://example.org/Pizza", "http://example.org/Food")?
    .build()?;

let json = ontology.to_json()?;
let restored = Ontology::from_json(&json)?;
```

`Ontology::from_file` returns an error until v0.2 parser support.

## CLI (v0.2+)

The CLI is wired but requires parser and engine implementations:

```bash
ontologos profile ontology.owl   # v0.2
ontologos classify ontology.owl  # v0.5
```

## Python API (v0.9+)

Python bindings are stubbed; file loading requires v0.2.

## Documentation

- [ROADMAP.md](ROADMAP.md) — versioned release plan (0.1 → 1.0 → 2.0)
- [CHANGELOG.md](CHANGELOG.md) — release history
- [SPEC.md](SPEC.md) — technical specification
- [PLAN.md](PLAN.md) — background and ecosystem vision
- [docs/research/](docs/research/) — OWL 2 and reasoner architecture notes

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
