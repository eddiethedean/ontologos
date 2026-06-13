# First Ontology

This guide walks through building a small ontology with the builder API.

## Prerequisites

- Rust 1.88+
- Clone of the [OntoLogos repository](https://github.com/eddiethedean/ontologos) (for examples), or use the [crates.io quick start](../getting-started/index.md#cratesio-only-no-clone)

Unfamiliar with OWL terms (TBox, ABox, materialization)? See the [Glossary](../guides/glossary.md).

> **Note:** `ontologos_core::Reasoner::classify()` is a facade stub. For file-based workflows use **CLI** (`ontologos classify`), **Python** (`Reasoner(path=...).classify()`), **`ontologos_facade::classify`**, or profile crates (`ElClassifier`, `RlEngine`, `RdfsEngine`). See [Choosing an API](../guides/choosing-an-api.md).

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

## ABox axioms (v0.4)

The builder supports named individuals and ABox assertions:

```rust
use ontologos_core::{Error, Ontology};

fn main() -> Result<(), Error> {
    let ontology = Ontology::builder()
        .class("http://example.org/Person")?
        .individual("http://example.org/alice")?
        .object_property("http://example.org/knows")?
        .individual("http://example.org/bob")?
        .class_assertion("http://example.org/alice", "http://example.org/Person")?
        .object_property_assertion(
            "http://example.org/alice",
            "http://example.org/knows",
            "http://example.org/bob",
        )?
        .same_individual(&[
            "http://example.org/alice",
            "http://example.org/alice-copy",
        ])?
        .different_individuals(&[
            "http://example.org/alice",
            "http://example.org/bob",
        ])?
        .build()?;

    let alice = ontology
        .lookup_entity("http://example.org/alice")
        .expect("alice");
    let person = ontology
        .lookup_entity("http://example.org/Person")
        .expect("person");
    assert_eq!(ontology.individuals_of(person), &[alice]);

    Ok(())
}
```

Additional builder methods: `equivalent_object_properties`, `asymmetric_object_property`, `property_domain`, `property_range`, `subproperty_of`.

After building an RL-shaped ontology, saturate with [OWL RL saturation](owl-rl-saturation.md).

## Next steps

- [Load an OWL file](load-owl-file.md) — parse real ontologies
- [OWL RL saturation](owl-rl-saturation.md) — forward-chaining on ABox + TBox
- [Profile detection](../guides/profile-detection.md)
- [JSON snapshots](../json-snapshot-v2.md)
- [Error reference](../reference/errors.md)
