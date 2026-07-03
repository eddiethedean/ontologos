# Rust integration contract

Canonical rules for embedding OntoLogos in Rust applications on **v1.0.0**. Other guides link here instead of repeating warnings.

Install pins: [Install and channels](install-channels.md). API picker: [Choosing an API](choosing-an-api.md).

## The contract

| Task | Use | Do **not** use |
|------|-----|----------------|
| Load OWL/RDF files | `ontologos_parser::load_ontology` | `Ontology::from_file` (returns `ParseNotAvailable`) |
| Multi-profile classify | `ontologos_facade::classify(&mut reasoner)` | `reasoner.classify()` on `ontologos_core::Reasoner` |
| Consistency (production) | `ontologos_facade::check_consistency` | `Reasoner::is_consistent()` on core |
| Single known profile | Profile crate directly (see below) | Facade when you only need one engine and want minimal deps |
| CLI / Python parity | Same facade functions the bindings call | Core engine stubs |

## Why `Reasoner` does not classify

`ontologos_core::Reasoner` holds **configuration and ontology state** (`Profile`, `ReasonerConfig`, incremental session). Engines live in separate crates (`ontologos-el`, `ontologos-rl`, `ontologos-dl`, …) to avoid circular dependencies and keep dependency graphs small.

Calling `Reasoner::classify()` on core returns an error in 1.0.0 — this is intentional, not a missing feature.

!!! info "The one rule"
    Build `Reasoner` with `Reasoner::builder()` → call **`ontologos_facade::classify(&mut reasoner)`**.

## Minimal example

```rust
use ontologos_core::{Profile, Reasoner};
use ontologos_facade::{classify, check_consistency, ClassifyOutcome};
use ontologos_parser::load_ontology;

let ontology = load_ontology("family.owl".as_ref())?;
let mut reasoner = Reasoner::builder()
    .profile(Profile::Auto)
    .build(ontology)?;

let consistency = check_consistency(&reasoner)?;
if consistency.complete && consistency.consistent {
    match classify(&mut reasoner)? {
        ClassifyOutcome::Taxonomy(t) => {
            println!("subsumptions: {}", t.subsumption_count());
        }
        ClassifyOutcome::Rdfs(r) | ClassifyOutcome::Rl(r) => {
            println!("inferred: {}", r.inferred_total());
        }
    }
}
```

`Cargo.toml`:

```toml
[dependencies]
ontologos-core = "1.0.0"
ontologos-parser = "1.0.0"
ontologos-facade = "1.0.0"
```

## When to skip the facade

Use a **profile crate directly** when the profile is fixed and you want fewer dependencies:

| Profile | Crate | Entry point |
|---------|-------|-------------|
| RDFS | `ontologos-rl` | `ontologos_rl::rdfs::RdfsEngine::materialize` |
| OWL RL | `ontologos-rl` | `RlEngine::saturate` |
| OWL EL | `ontologos-el` | `ElClassifier::classify` |
| OWL DL | `ontologos-dl` | `DlClassifier::classify` (or facade for routing) |

Use the **facade** for `Profile::Auto`, `Dl`, `Alc`, `Swrl`, or when mirroring CLI/Python behavior.

## Consistency checks

Prefer `check_consistency` — it returns `{ consistent, complete }`. When `complete` is `false` (DL budget exhausted), do not treat `consistent` as proof.

```rust
let result = ontologos_facade::check_consistency(&reasoner)?;
if !result.complete {
    // retry with higher budget_secs or fail closed
}
```

Set `ReasonerConfig::budget_secs` or `ONTOLOGOS_DL_BUDGET_SECS` for DL wall-clock limits.

## Untrusted input

- OWL files: `load_ontology_in` / `load_ontology_with_limits_and_base` — see [Production integration](production-integration.md)
- JSON: `Ontology::from_json_with_limits` — see [Security](../security.md)

## Related

- [Classify quick start](../getting-started/classify-quickstart.md)
- [Facade API guide](facade-api.md)
- [Facade API reference](../reference/facade.md)
- [Errors](../reference/errors.md)
