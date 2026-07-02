# Production Integration

Patterns for embedding OntoLogos in services and pipelines.

--8<-- "snippets/channel-banner.md"

## Dependency selection

| Need | Minimum crates |
|------|----------------|
| Build/load JSON only | `ontologos-core` |
| Load OWL files | `+ ontologos-parser` |
| RDFS materialization | `+ ontologos-rdfs` |
| OWL RL saturation | `+ ontologos-rl` |
| OWL EL taxonomy | `+ ontologos-el`, `+ ontologos-ql` |
| Profile routing | `+ ontologos-profile` |
| Explanations | `+ ontologos-explain` |

See [Choosing an API](choosing-an-api.md). There is no umbrella `ontologos` meta-crate on crates.io. For Python services, use `pip install ontologos`.

## Untrusted OWL uploads

Do **not** use `load_ontology(path)` for user-supplied paths without constraints.

```rust
use ontologos_parser::{load_ontology_with_limits_and_base, ParseLimits};
use std::path::Path;

let base = Path::new("/var/uploads/sandbox");
let user_file = Path::new("ontology.owl");

let limits = ParseLimits {
    max_file_bytes: 1_048_576, // 1 MiB
    ..ParseLimits::default()
};

let ontology = load_ontology_with_limits_and_base(user_file, limits, Some(base))?;
```

`load_ontology_with_limits_and_base` canonicalizes paths and rejects directory traversal outside `base`. See [Security](../security.md).

## Untrusted JSON snapshots

Use `from_json_with_limits` — format v1 is rejected; v2 keys axioms by IRI string.

```rust
use ontologos_core::{Limits, Ontology};

let limits = Limits {
    max_json_bytes: 1_048_576,
    ..Limits::default()
};
let ontology = Ontology::from_json_with_limits(json_bytes, limits)?;
```

## Persisting results

After materialization or saturation, persist the enriched ontology:

```rust
let json = ontology.to_json()?;
std::fs::write("saturated.json", json)?;
```

Reload later with `Ontology::from_json`. OWL export is not built in — keep JSON v2 or retain the source OWL plus processing metadata.

## Reasoning workflow (Rust)

Use profile-aware routing instead of manual `match` on detected profiles:

```rust
use ontologos_core::{Profile, Reasoner, ReasonerConfig};
use ontologos_facade::{check_consistency, classify, ClassifyOutcome};
use ontologos_parser::load_ontology;

let ontology = load_ontology(path)?;
let mut reasoner = Reasoner::builder()
    .profile(Profile::Auto)
    .config(ReasonerConfig {
        incremental: true,
        ..ReasonerConfig::default()
    })
    .build(ontology)?;

let consistency = check_consistency(&reasoner)?;
if !consistency.complete || !consistency.consistent {
    return Err("ontology inconsistent or consistency check incomplete".into());
}

match classify(&mut reasoner)? {
    ClassifyOutcome::Taxonomy(t) => { /* EL or DL taxonomy */ }
    ClassifyOutcome::Rl(r) => { /* RL saturation report */ }
    ClassifyOutcome::Rdfs(r) => { /* RDFS materialization report */ }
}
```

For direct engine access (no `Reasoner` wrapper):

- RDFS: `ontologos_rdfs::RdfsEngine::materialize` or `classify_reasoner`
- RL: `ontologos_rl::RlEngine::saturate` or `classify_reasoner`
- EL: `ontologos_el::ElClassifier::classify` or `ontologos_el::classify_reasoner`

> **Note:** Classification and consistency live in **`ontologos_facade`** — not on `ontologos_core::Reasoner`. See [Facade API](facade-api.md) and [Facade reference](../reference/facade.md).

When merging triples from reasonable back into core, apply [merge limits](../security.md#reasoning-merge-limits-v090) to cap axiom growth on untrusted input.

## Python services

v0.9.0 Python returns structured report dicts from `classify()`, supports `explain()`, incremental mutations, and optional DataFrame export:

```python
from ontologos import Reasoner

reasoner = Reasoner(path="/data/ontology.owl", profile="auto", incremental=True)
report = reasoner.classify()
graph = reasoner.explain()
reasoner.add_subclass_of("http://example.org/A", "http://example.org/B")
reasoner.classify()
```

`Reasoner` is **not thread-safe** — one instance per worker or external locking.

Positional `Reasoner("file.owl")` still works; prefer keyword `path=` for clarity.

See [Python guide](python.md) and [v0.8→v0.9 migration](../migration/v0.8.x-to-v0.9.0.md).

## Incremental pipelines

Enable `ReasonerConfig::incremental` (Rust) or `incremental=True` (Python) when ontologies change between passes. After axiom removal, engines strip inferred axioms before rematerialization — see [Incremental reasoning](incremental-reasoning.md).

CLI: `ontologos classify --incremental ontology.owl` runs a single incremental pass per invocation (library multi-pass workflows hold session state across edits).

## Observability

Inspect `parse_meta` after load:

- `warnings` — skipped mapping shapes
- `mapped_axiom_count` / `skipped_axiom_count` — see [Protégé vs counts](protege-axiom-counts.md)

Engine reports (`MaterializationReport`, EL taxonomy, proof graphs) expose counts, clashes (RL), and subsumptions (EL). RDFS/RL rule-level telemetry may be empty when delegating to reasonable — see [Reasonable adapter limits](../reference/reasonable-limits.md).

## Related

- [Security](../security.md)
- [Performance](performance.md)
- [Comparison](../comparison.md)
- [Incremental reasoning](incremental-reasoning.md)
