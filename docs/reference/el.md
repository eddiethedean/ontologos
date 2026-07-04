# EL API Reference

OWL EL classification via [`ontologos-el`](https://docs.rs/ontologos-el/1.1.0).

Tutorial: [OWL EL classification](../getting-started/owl-el-classification.md).

## Primary types

| Type | Role |
|------|------|
| `ElClassifier` | Stateless EL completion engine |
| `ElReport` | Classification report with timing metadata |

## Functions

| Function | Description |
|----------|-------------|
| `ElClassifier::new()` | Create classifier |
| `ElClassifier::classify(&Ontology)` | Return `Taxonomy` |
| `classify_reasoner(&mut Reasoner)` | Classify via reasoner session (incremental-aware) |

## Errors (`ontologos_el::Error`)

| Variant | Meaning |
|---------|---------|
| `WrongProfile` | Ontology or config is not EL-shaped |
| `Core` | Underlying `ontologos_core::Error` |

## Example

```rust
use ontologos_el::ElClassifier;

let taxonomy = ElClassifier::new().classify(&ontology)?;
println!("{}", taxonomy.subsumption_count());
```

For file-based Pizza demos, fetch the corpus — see [OWL EL classification](../getting-started/owl-el-classification.md#prerequisites).

## docs.rs

Full API: [docs.rs/ontologos-el/1.1.0](https://docs.rs/ontologos-el/1.1.0)

## Related

- [Facade API](facade.md) — `Profile::El` routing
- [Query API](query.md) · [QL API](ql.md)
