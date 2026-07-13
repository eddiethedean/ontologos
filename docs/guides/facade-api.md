# Facade API (`ontologos-facade`)

The **`ontologos-facade`** crate is the unified routing layer for CLI, Python, and multi-profile Rust apps. It avoids circular dependencies between EL and DL engines while exposing one `classify()` entry point.

!!! note "Workspace crate"
    `ontologos-facade` is published on crates.io in the workspace release set. Prefer **`ontologos_facade::classify`** over profile-crate helpers when you need **DL**, **ALC**, or **SWRL** routing.

## When to use the facade

| Use facade | Use profile crates directly |
|------------|----------------------------|
| `Profile::Auto` (including DL-detected ontologies) | Single known profile (e.g. only RDFS) |
| `Profile::Dl`, `Profile::DlPreview`, `Alc`, `Swrl` | Embedding reasonable/EL internals |
| CLI/Python parity in Rust tests | Minimal dependency footprint |

## Rust API

```rust
use ontologos_core::{Profile, Reasoner};
use ontologos_facade::{self, ClassifyOutcome};
use ontologos_parser::load_ontology;

let ontology = load_ontology(path)?;
let mut reasoner = Reasoner::builder()
    .profile(Profile::Auto)
    .build(ontology)?;

match ontologos_facade::classify(&mut reasoner)? {
    ClassifyOutcome::Taxonomy(t) => { /* EL, DL, ALC */ }
    ClassifyOutcome::Rdfs(report) => { /* RDFS materialization */ }
    ClassifyOutcome::Rl(report) => { /* RL saturation */ }
}
```

### Consistency check (production API)

Use **`check_consistency`** when you need to know whether a check finished. When `complete` is `false`, do not treat `consistent` as a proof.

```rust
let result = ontologos_facade::check_consistency(&reasoner)?;
if !result.complete {
    // budget or tableau limit — retry with higher budget_secs or fail closed
}
if result.consistent {
    // proved consistent
}
```

`ConsistencyResult` implements `Serialize` for logging and JSON export.

**Convenience bool:** `is_consistent` returns `Ok(true|false)` only when the check is complete; it returns `Err(IncompleteConsistency)` on DL budget limits. Prefer `check_consistency` in services.

```rust
// OK for scripts; fragile for DL with tight budgets
let consistent = ontologos_facade::is_consistent(&reasoner)?;
```

Set a wall-clock budget via `ReasonerConfig::budget_secs` (or `ONTOLOGOS_DL_BUDGET_SECS` env fallback in DL).

### Entailment

Typed checks (matches CLI `entail` JSON):

```rust
use ontologos_facade::{is_entailed_axiom, EntailmentCheck};

let entailed = is_entailed_axiom(
    &mut reasoner,
    EntailmentCheck::SubClassOf {
        sub: "http://ex.org/A".into(),
        sup: "http://ex.org/B".into(),
    },
)?;
```

String overloads `is_subsumption_entailed` / `is_entailed` are aliases for `SubClassOf` only.

### Property lookups

```rust
let subs = ontologos_facade::get_sub_object_properties(&reasoner, property_iri, direct)?;
let values = ontologos_facade::get_object_property_values(&reasoner, subject_iri, property_iri)?;
```

### Taxonomy navigation

`taxonomy_hierarchy(ontology, taxonomy)` returns a hierarchy navigator — not the OWL QL conjunctive query API (`ontologos_ql` / CLI `query`). `query_engine` is a deprecated alias.

### Profile routing

| `Profile` | Routed to |
|-----------|-----------|
| `Auto` | EL/RL if detected; **DL** if profile detection returns DL |
| `El`, `Rdfs`, `Rl` | `ontologos-el` router (EL/RL/RDFS paths) |
| `Alc` | `ontologos-alc::classify` |
| `Dl` | `ontologos-dl::classify` |
| `DlPreview` | `ontologos-dl::classify` with preview gating (same engine, explicit preview mode) |
| `Swrl` | `ontologos-swrl::classify_with_swrl` |

Preview limitations: [Preview profiles](preview-profiles.md).

## What not to do

```rust
// DON'T — classification is not on ontologos_core::Reasoner
// reasoner.classify()?;  // removed — use ontologos_facade::classify

// DON'T — always fails; use ontologos_parser::load_ontology
Ontology::from_file(path)?;

// DON'T — EL-only helpers skip full DL hybrid for Profile::Dl
ontologos_el::classify_reasoner(&mut reasoner)?; // when profile is Dl
```

Use **`ontologos_facade::classify`** or the profile-specific engine crate.

## Loading untrusted ontologies

Default `load_ontology` is permissive. For production paths or user uploads:

```rust
use ontologos_parser::{load_ontology_with_limits, ParseLimits};

let limits = ParseLimits {
    max_file_bytes: 10 * 1024 * 1024,
    max_entities: 50_000,
    max_axioms: 100_000,
    merge_imports: false,
    ..ParseLimits::default()
};
let ontology = load_ontology_with_limits(path, limits)?;
```

See `ParseLimits` on [docs.rs/ontologos-parser](https://docs.rs/ontologos-parser/1.1.4) for byte/entity/axiom caps.

## Dependencies

```toml
[dependencies]
ontologos-core = "1.1.4"
ontologos-parser = "1.1.4"
ontologos-facade = "1.1.4"
```

On **`main`** (workspace **1.1.4**, pre-tag), use `"1.1.4"` pins or path/git dependencies after the release is published. The facade pulls in `ontologos-el`, `ontologos-dl`, `ontologos-alc`, `ontologos-swrl`, and `ontologos-rl` (including RDFS) transitively.

## Related

- [Choosing an API](choosing-an-api.md)
- [Preview profiles](preview-profiles.md)
- [Architecture](../architecture.md)
- [docs.rs/ontologos-facade](https://docs.rs/ontologos-facade/1.1.4)
