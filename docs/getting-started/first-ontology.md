# First Ontology

This guide walks through building a small ontology with the v0.1 API.

## Prerequisites

- Rust 1.78+
- Clone of the [OntoLogos repository](https://github.com/eddiethedean/ontologos)

## Run the example

```bash
cargo run -p ontologos-core --example pizza_builder
```

## Step by step

```rust
use ontologos_core::{Error, Ontology};

fn main() -> Result<(), Error> {
    // 1. Register entities and axioms via the builder
    let ontology = Ontology::builder()
        .class("http://example.org/Pizza")?
        .class("http://example.org/Food")?
        .subclass_of("http://example.org/Pizza", "http://example.org/Food")?
        .build()?;

    // 2. Query the taxonomy index
    let pizza = ontology
        .lookup_entity("http://example.org/Pizza")
        .expect("registered");
    let supers = ontology.direct_superclasses(pizza);
    assert_eq!(supers.len(), 1);

    // 3. Round-trip through JSON v2
    let json = ontology.to_json()?;
    let restored = Ontology::from_json(&json)?;
    assert_eq!(restored, ontology);

    Ok(())
}
```

## Next steps

- [JSON snapshots](../json-snapshot-v2.md) — hand-author or load snapshot files
- [Error reference](../reference/errors.md) — interpret failures
- [ROADMAP](../../ROADMAP.md) — OWL file loading in v0.2
