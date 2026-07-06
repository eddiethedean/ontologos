# Production Integration

Patterns for embedding OntoLogos in services and pipelines. Profile status: [Profile stability matrix](profile-stability.md).

## Dependency selection

| Need | Minimum crates |
|------|----------------|
| Build/load JSON only | `ontologos-core` |
| Load OWL files | `+ ontologos-parser` |
| RDFS materialization | `ontologos-rl` (`rdfs` module) |
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

Use `from_json_with_limits` — format v1 is rejected. Writers emit **v3** on v1.1.1; readers accept v2 and v3.

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

Reload later with `Ontology::from_json`. OWL export is not built in — keep JSON snapshots or retain the source OWL plus processing metadata. See [JSON snapshot v3](../json-snapshot-v3.md).

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

## OWL DL in production

Stable **`--profile dl`** on v1.1.1 uses bounded tableau reasoning. Follow this checklist before serving DL in production:

1. **Always use `check_consistency`** — not `is_consistent`. Inspect `ConsistencyResult { consistent, complete }`. When `complete == false`, the check hit a wall-clock or tableau budget; do not treat the ontology as proven consistent.
2. **Set a wall-clock budget** — `ReasonerConfig { budget_secs: Some(30), .. }` or `ONTOLOGOS_DL_BUDGET_SECS`. Without a budget, DL consistency may run until natural completion (unbounded on pathological inputs).
3. **Never set conformance env vars** — leave `ONTOLOGOS_CONFORMANCE`, `ONTOLOGOS_STRICT_TAXONOMY`, and `ONTOLOGOS_CI_PROMOTED_ONLY` unset. CI shortcuts are not production semantics. See [Security](../security.md#conformance-harness-environment-do-not-set-in-production).
4. **Validate on your corpus** — HermiT catalog parity (`parity_pct = 100%`) applies to gated test corpora only, not every real-world ontology. Run classify + consistency on your files before cutover.
5. **Single-thread OWL loads** — horned-owl parsing is serialized by a process-wide mutex; use one load at a time or process-isolated workers. See [Security](../security.md#parser-concurrency-server-embedders).

### Rust (DL service)

```rust
use ontologos_core::{Profile, Reasoner, ReasonerConfig};
use ontologos_facade::{check_consistency, classify, ClassifyOutcome};
use ontologos_parser::{load_ontology_in, ParseLimits};
use std::path::Path;

let base = Path::new("/var/uploads/sandbox");
let ontology = load_ontology_in(
    base,
    Path::new("ontology.owl"),
)?;

let mut reasoner = Reasoner::builder()
    .profile(Profile::Dl)
    .config(ReasonerConfig {
        budget_secs: Some(30),
        ..ReasonerConfig::default()
    })
    .build(ontology)?;

let result = check_consistency(&reasoner)?;
if !result.complete {
    return Err("DL consistency incomplete — increase budget_secs".into());
}
if !result.consistent {
    return Err("ontology inconsistent".into());
}

match classify(&mut reasoner)? {
    ClassifyOutcome::Taxonomy(t) => { /* use t */ }
    _ => unreachable!("Profile::Dl yields Taxonomy"),
}
```

### Python (DL service)

```python
from ontologos import Reasoner

reasoner = Reasoner(
    path="/data/ontology.owl",
    profile="dl",
    budget_secs=30,
)
consistency = reasoner.check_consistency()
if not consistency["complete"]:
    raise RuntimeError("DL consistency incomplete — increase budget_secs")
if not consistency["consistent"]:
    raise RuntimeError("ontology inconsistent")
report = reasoner.classify()
```

CLI: `ontologos classify --profile dl --budget-secs 30 ontology.owl`

See [Facade API](facade-api.md) · [Performance](performance.md) · [Evaluator scope](evaluator-scope.md).

For direct engine access (no `Reasoner` wrapper):

- RDFS: `ontologos_rl::rdfs::RdfsEngine::materialize` or `ontologos_rl::rdfs::classify_reasoner`
- RL: `ontologos_rl::RlEngine::saturate` or `classify_reasoner`
- EL: `ontologos_el::ElClassifier::classify` or `ontologos_el::classify_reasoner`

> **Note:** Classification and consistency live in **`ontologos_facade`** — not on `ontologos_core::Reasoner`. See [Facade API](facade-api.md) and [Facade reference](../reference/facade.md).

When merging triples from reasonable back into core, apply [merge limits](../security.md#reasoning-merge-limits-v090) to cap axiom growth on untrusted input.

## Python services

Python **1.0.0** returns structured report dicts from `classify()`, supports `explain()`, incremental mutations, and optional DataFrame export:

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

See [Python guide](python.md) and [v0.9.x → v1.0.0 migration](../migration/v0.9.x-to-v1.0.0.md).

## Incremental pipelines

Enable `ReasonerConfig::incremental` (Rust) or `incremental=True` (Python) when ontologies change between passes. After axiom removal, engines strip inferred axioms before rematerialization — see [Incremental reasoning](incremental-reasoning.md).

CLI: `ontologos classify --incremental ontology.owl` runs a single incremental pass per invocation (library multi-pass workflows hold session state across edits).

## Observability

For containers, health checks, DL budgets, and `tracing` setup, see [Deployment and observability](deployment.md).

Inspect `parse_meta` after load:

- `warnings` — skipped mapping shapes
- `mapped_axiom_count` / `skipped_axiom_count` — see [Protégé vs counts](protege-axiom-counts.md)

Engine reports (`MaterializationReport`, EL taxonomy, proof graphs) expose counts, clashes (RL), and subsumptions (EL). RDFS/RL rule-level telemetry may be empty when delegating to reasonable — see [Reasonable adapter limits](../reference/reasonable-limits.md).

## Related

- [Deployment](deployment.md)
- [Security](../security.md)
- [Performance](performance.md)
- [Comparison](../comparison.md)
- [Incremental reasoning](incremental-reasoning.md)
