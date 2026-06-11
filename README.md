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

## Quick start

```bash
# Build the workspace
cargo build

# Run the CLI
cargo run -p ontologos-cli -- profile path/to/ontology.owl

# Run tests
cargo test --workspace
```

## CLI

```bash
ontologos profile ontology.owl
ontologos classify ontology.owl
ontologos materialize ontology.owl
ontologos explain ontology.owl
```

Output formats: `--format text|json|yaml` (yaml pending).

## Rust API

```rust
use ontologos_core::{Ontology, Profile, Reasoner};

let ontology = Ontology::from_file("pizza.owl")?;
let reasoner = Reasoner::builder()
    .profile(Profile::Auto)
    .build(ontology)?;
reasoner.classify()?;
```

## Python API

```python
from ontologos import Reasoner

r = Reasoner("ontology.owl")
r.classify()
```

## Documentation

- [ROADMAP.md](ROADMAP.md) — phased delivery plan and current status
- [SPEC.md](SPEC.md) — technical specification
- [PLAN.md](PLAN.md) — background and ecosystem vision

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
