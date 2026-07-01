# ontologos-core

In-memory OWL ontology model: interned IRIs, typed axioms, JSON v2 snapshots, and `Reasoner` builder.

**Docs:** [ontologos.readthedocs.io](https://ontologos.readthedocs.io/en/latest/) · [docs.rs](https://docs.rs/ontologos-core/0.9.0)

## Install

```toml
[dependencies]
ontologos-core = "0.9.0"
```

## Quick start

```rust
use ontologos_core::Ontology;

let ontology = Ontology::builder()
    .class("http://example.org/A")?
    .class("http://example.org/B")?
    .subclass_of("http://example.org/A", "http://example.org/B")?
    .build()?;
```

For file loading use `ontologos-parser`. For classification use `ontologos-facade` or profile crates — not `Reasoner::classify()` (deprecated).

## Example

```bash
cargo run -p ontologos-core --example pizza_builder
```
