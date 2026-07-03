# Core API Reference

In-memory ontology model via [`ontologos-core`](https://docs.rs/ontologos-core/1.0.0).

Narrative: [First ontology](../getting-started/first-ontology.md) · [JSON snapshot v3](../json-snapshot-v3.md).

!!! warning "Classification is not on core"
    Use [`ontologos_facade::classify`](facade.md) or profile crates — not `Reasoner::classify()` (removed in 1.0.0).

## `Ontology`

Central embed-facing representation.

| Method / type | Description |
|---------------|-------------|
| `Ontology::builder()` | Start `OntologyBuilder` |
| `Ontology::default()` | Empty ontology |
| `to_json()` / `from_json()` | JSON snapshot v3 (v2 read supported) |
| `from_json_with_limits(bytes, Limits)` | Deserialize with resource caps (untrusted input) |
| `axiom_count()` | Mapped axiom count (not Protégé totals) |
| `entity_count()` | Entity registry size |

`Ontology::from_file` returns `ParseNotAvailable` — use [`ontologos_parser::load_ontology`](parser.md).

## `OntologyBuilder`

Fluent construction for common TBox/ABox axioms:

```rust
use ontologos_core::Ontology;

let ontology = Ontology::builder()
    .class("http://example.org/Pizza")?
    .class("http://example.org/Food")?
    .subclass_of("http://example.org/Pizza", "http://example.org/Food")?
    .build()?;
```

See builder methods on [docs.rs](https://docs.rs/ontologos-core/1.0.0/ontologos_core/struct.OntologyBuilder.html).

## `Reasoner` and `ReasonerBuilder`

Configuration holder for profile routing — engines run via facade or profile crates.

```rust
use ontologos_core::{Profile, Reasoner, ReasonerConfig};

let reasoner = Reasoner::builder()
    .profile(Profile::Auto)
    .config(ReasonerConfig {
        incremental: true,
        budget_secs: Some(30),
        ..ReasonerConfig::default()
    })
    .build(ontology)?;
```

### `ReasonerConfig`

| Field | Default | Description |
|-------|---------|-------------|
| `incremental` | `false` | Reuse session state across axiom mutations |
| `explanations` | `false` | Record inference traces where supported |
| `parallelism` | `1` | DL tableau workers (`1..=64`) |
| `budget_secs` | `None` | DL wall-clock budget; falls back to `ONTOLOGOS_DL_BUDGET_SECS` |

### `Profile`

| Variant | Routed engine |
|---------|---------------|
| `Auto` | Detected EL/RL/RDFS or DL hybrid |
| `Rdfs`, `Rl`, `El` | Respective profile crate |
| `Dl`, `DlPreview` | `ontologos-dl` |
| `Alc` | `ontologos-alc` |
| `Swrl` | `ontologos-swrl` |

Canonical stability: [Profile stability matrix](../guides/profile-stability.md).

## `Taxonomy`

EL/DL classification output — subsumption pairs and counts. Produced by `ElClassifier`, `ontologos-dl`, or `ontologos_facade::classify` (`ClassifyOutcome::Taxonomy`).

## `Limits`

Resource caps for JSON deserialization (`from_json_with_limits`):

| Field | Default (approx.) |
|-------|-------------------|
| `max_json_bytes` | 16 MiB |
| `max_entities` | 1_000_000 |
| `max_axioms` | 10_000_000 |
| `max_iri_len` | 8_192 |
| `max_class_operands` | 10_000 |
| `max_literal_bytes` | 1 MiB |

## Sessions and incremental reasoning

`ReasonerSession` and `OntologyRevision` track dirty axioms for incremental re-classification. See [Incremental reasoning](../guides/incremental-reasoning.md).

## Related types

| Type | Role |
|------|------|
| `Axiom`, `AxiomStore`, `AxiomIndex` | Structured axiom storage |
| `EntityRegistry`, `EntityId`, `EntityKind` | Typed entities |
| `InternPool`, `IriId` | Deduplicated IRIs |
| `ParseMeta` | Parser scan metadata on loaded ontologies |
| `ConsistencyResult` | `{ consistent, complete }` from facade checks |

## Related

- [Facade API](facade.md)
- [Parser API](parser.md)
- [Error reference](errors.md)
- [Supported constructs](supported-constructs.md)
