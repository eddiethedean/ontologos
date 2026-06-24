# Query API Reference

Taxonomy queries over classified ontologies via [`ontologos-query`](https://docs.rs/ontologos-query/0.9.0).

## Overview

`QueryEngine` provides hierarchy navigation over an EL classification `Taxonomy`. It is typically used after `ElClassifier::classify` or CLI/Python EL classification.

## Rust API

```rust
use ontologos_el::ElClassifier;
use ontologos_parser::load_ontology;
use ontologos_query::QueryEngine;

let ontology = load_ontology(path)?;
let taxonomy = ElClassifier::new().classify(&ontology)?;
let engine = QueryEngine::new(&ontology, &taxonomy);

let class = engine.lookup("http://example.org/Pizza").unwrap();
let subs = engine.direct_subclasses(class)?;
let supers = engine.direct_superclasses(class)?;
let subsumed = engine.is_subsumed(sub, sup)?;
let equiv = engine.equivalent_classes(class)?;
let unsat = engine.unsatisfiable_classes();
```

### Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `new(ontology, taxonomy)` | `QueryEngine` | Build petgraph-backed hierarchy view |
| `lookup(iri)` | `Option<EntityId>` | Resolve class IRI to entity ID |
| `direct_subclasses(class)` | `Vec<EntityId>` | Immediate subclasses in taxonomy |
| `direct_superclasses(class)` | `Vec<EntityId>` | Immediate superclasses |
| `is_subsumed(sub, sup)` | `bool` | Whether `sub ⊑ sup` in taxonomy |
| `equivalent_classes(class)` | `Option<Vec<EntityId>>` | Equivalence cluster containing class |
| `unsatisfiable_classes()` | `Vec<EntityId>` | Classes equivalent to `owl:Nothing` |

### Errors

| Error | Cause |
|-------|-------|
| `UnknownEntity` | Entity ID not a known class |

## Python

After EL classification, use the taxonomy dict from `classify()` or the `reasoner.taxonomy` property:

```python
from ontologos import Reasoner

reasoner = Reasoner(path="pizza.owl", profile="el")
taxonomy = reasoner.classify()
for sub, sup in taxonomy["subsumptions"][:5]:
    print(sub, "subClassOf", sup)
```

Optional DataFrame export:

```python
from ontologos.export import subsumptions_to_pandas

df = subsumptions_to_pandas(taxonomy)
```

Requires `pip install 'ontologos[pandas]'`.

## Related

- [OWL EL classification](../getting-started/owl-el-classification.md)
- [Python guide](../guides/python.md)
- [docs.rs/ontologos-query](https://docs.rs/ontologos-query/0.9.0)
