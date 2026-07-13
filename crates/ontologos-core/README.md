# ontologos-core

In-memory OWL ontology model: interned IRIs, typed axioms, JSON v3 snapshots (v2 readable), and `Reasoner` builder.

**Docs:** [ontologos.readthedocs.io](https://ontologos.readthedocs.io/en/latest/) · [docs.rs](https://docs.rs/ontologos-core/1.1.4)

## Install

```toml
[dependencies]
ontologos-core = "1.1.4"
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
