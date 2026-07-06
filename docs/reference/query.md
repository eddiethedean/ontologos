# Query API Reference

Taxonomy navigation and conjunctive queries via [`ontologos-ql`](https://docs.rs/ontologos-ql/1.1.2).

The `ontologos-query` crate is a **deprecated shim** — depend on `ontologos-ql` directly.

## Overview

| API | Purpose |
|-----|---------|
| [`TaxonomyHierarchy`](https://docs.rs/ontologos-ql/1.1.2/ontologos_ql/struct.TaxonomyHierarchy.html) | Subclass/superclass navigation after EL or DL classification |
| [`answer_query`](https://docs.rs/ontologos-ql/1.1.2/ontologos_ql/fn.answer_query.html) | OWL QL conjunctive query answering |

Use after `ElClassifier::classify`, `ontologos_facade::classify`, or CLI/Python classification.

## Hierarchy navigation (Rust)

```rust
use ontologos_el::ElClassifier;
use ontologos_parser::load_ontology;
use ontologos_ql::TaxonomyHierarchy;

let ontology = load_ontology(path)?;
let taxonomy = ElClassifier::new().classify(&ontology)?;
let hierarchy = TaxonomyHierarchy::new(&ontology, &taxonomy);

let class = hierarchy.lookup("http://example.org/Pizza").unwrap();
let subs = hierarchy.direct_subclasses(class)?;
let supers = hierarchy.direct_superclasses(class)?;
let subsumed = hierarchy.is_subsumed(sub, sup)?;
let instances = hierarchy.instances_of(class)?;
let types = hierarchy.types_of(individual)?;
```

### `TaxonomyHierarchy` methods

| Method | Returns | Description |
|--------|---------|-------------|
| `new(ontology, taxonomy)` | `TaxonomyHierarchy` | Build hierarchy view |
| `lookup(iri)` | `Option<EntityId>` | Resolve class IRI to entity ID |
| `direct_subclasses(class)` | `Vec<EntityId>` | Immediate subclasses in taxonomy |
| `direct_superclasses(class)` | `Vec<EntityId>` | Immediate superclasses |
| `is_subsumed(sub, sup)` | `bool` | Whether `sub ⊑ sup` in taxonomy |
| `instances_of(class)` | `Vec<EntityId>` | Individuals typed as class or subclass |
| `types_of(individual)` | `Vec<EntityId>` | Classes asserted on individual |

### Errors

| Error | Cause |
|-------|-------|
| `UnknownEntity` | Entity ID missing or not a class/individual |

## Conjunctive queries (Rust)

```rust
use ontologos_ql::{answer_query, parse_conjunctive_query};

let query = parse_conjunctive_query("Type(?x, http://ex.org/A)")?;
let answers = answer_query(&ontology, &taxonomy, &query)?;
```

See [CLI `query` subcommand](cli.md) and [QL reference](ql.md).

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
from ontologos import Reasoner, subsumptions_to_pandas

report = Reasoner(path="pizza.owl", profile="el").classify()
df = subsumptions_to_pandas(report)  # pip install 'ontologos[pandas]'
```

## Related

- [OWL EL classification](../getting-started/owl-el-classification.md)
- [QL reference](ql.md)
- [CLI reference](cli.md)
