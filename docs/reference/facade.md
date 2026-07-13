# Facade API Reference

Unified classification, consistency, and entailment via [`ontologos-facade`](https://docs.rs/ontologos-facade/1.1.4).

For narrative guidance see [Facade API guide](../guides/facade-api.md).

## Classification

```rust
use ontologos_core::{Profile, Reasoner};
use ontologos_facade::{classify, ClassifyOutcome};
use ontologos_parser::load_ontology;

let ontology = load_ontology(path)?;
let mut reasoner = Reasoner::builder()
    .profile(Profile::Auto)
    .build(ontology)?;

match classify(&mut reasoner)? {
    ClassifyOutcome::Taxonomy(t) => { /* EL, DL, ALC */ }
    ClassifyOutcome::Rdfs(report) => { /* RDFS materialization */ }
    ClassifyOutcome::Rl(report) => { /* RL saturation */ }
}
```

| Function | Returns | Description |
|----------|---------|-------------|
| `classify(&mut Reasoner)` | `ClassifyOutcome` | Route by configured profile |
| `taxonomy_from_outcome(&ClassifyOutcome)` | `Option<&Taxonomy>` | Extract taxonomy when present |

## Consistency

| Function | Returns | Description |
|----------|---------|-------------|
| `check_consistency(&Reasoner)` | `ConsistencyResult` | `{ consistent, complete }` — prefer in services |
| `is_consistent(&Reasoner)` | `Result<bool>` | `Err` when check did not complete (DL budget) |

Set `ReasonerConfig::budget_secs` or `ONTOLOGOS_DL_BUDGET_SECS` for DL wall-clock limits.

## Entailment

| Function | Description |
|----------|-------------|
| `is_entailed_axiom(&mut Reasoner, EntailmentCheck)` | Typed subsumption, class, or property assertion |
| `is_subsumption_entailed(&mut Reasoner, sub, sup)` | `SubClassOf` shorthand |
| `is_entailed(&mut Reasoner, sub, sup)` | Alias for subsumption |

## Property lookups

| Function | Description |
|----------|-------------|
| `get_sub_object_properties(&Reasoner, property_iri, direct)` | Sub-properties in hierarchy |
| `get_object_property_values(&Reasoner, subject_iri, property_iri)` | Object property fillers |

## Taxonomy navigation

| Function | Description |
|----------|-------------|
| `taxonomy_hierarchy(ontology, taxonomy)` | Build `ontologos_ql::TaxonomyHierarchy` |

## JSON helpers

| Function | Description |
|----------|-------------|
| `taxonomy_json(&Taxonomy, ontology)` | Serialize taxonomy |
| `rdfs_materialization_json(&MaterializationReport)` | RDFS report JSON |
| `rl_materialization_json(&MaterializationReport)` | RL report JSON |

## `ontologos_facade::Error`

Transparent wrapper over profile engine errors:

| Variant | Source |
|---------|--------|
| `El` | `ontologos_el::Error` |
| `Dl` | `ontologos_dl::Error` |
| `Rl` | `ontologos_rl::Error` |
| `Swrl` | `ontologos_swrl::Error` |
| `Core` | `ontologos_core::Error` |

DL-specific variants (via `Dl`): `Inconsistent`, `PreviewLimit`, `IncompleteConsistency`, `ResourceLimit`.

## What not to use

| API | Use instead |
|-----|-------------|
| `Ontology::from_file` | `ontologos_parser::load_ontology` |
| `Reasoner::classify()` on core | **Removed** — use `ontologos_facade::classify` |
| `Reasoner::is_consistent()` on core | **Removed** — use `ontologos_facade::check_consistency` |

## Related

- [Facade API guide](../guides/facade-api.md)
- [Error reference](errors.md)
- [CLI reference](cli.md)
- [Profile stability](../guides/profile-stability.md)
