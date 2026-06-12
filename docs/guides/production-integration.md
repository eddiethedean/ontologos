# Production Integration

Patterns for embedding OntoLogos in services and pipelines. v0.4 is **pre-release** — validate against your ontologies before production deployment.

## Dependency selection

| Need | Minimum crates |
|------|----------------|
| Build/load JSON only | `ontologos-core` |
| Load OWL files | `+ ontologos-parser` |
| RDFS materialization | `+ ontologos-rdfs` |
| OWL RL saturation | `+ ontologos-rl` |
| Profile routing | `+ ontologos-profile` |

See [Choosing an API](choosing-an-api.md). There is no umbrella `ontologos` meta-crate.

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

Reload later with `Ontology::from_json`. OWL export is not built into v0.4 — keep JSON v2 or retain the source OWL plus processing metadata.

## Reasoning workflow

```rust
use ontologos_parser::load_ontology;
use ontologos_profile::detect_profile;
use ontologos_rl::RlEngine;

let mut ontology = load_ontology(path)?;
let profile = detect_profile(&ontology)?;

// Route manually in v0.4 — Auto profile on Reasoner is not implemented
match profile.detected {
    Some(p) if format!("{p:?}").contains("Rl") => {
        RlEngine::new(1).saturate(&mut ontology)?;
    }
    _ => {
        ontologos_rdfs::RdfsEngine::new().materialize(&mut ontology)?;
    }
}
```

Automatic profile routing ships in v0.5.

## Do not use `Reasoner::classify()` on core

Call profile crate helpers directly:

- RDFS: `ontologos_rdfs::RdfsEngine::materialize` or `classify_reasoner`
- RL: `ontologos_rl::RlEngine::saturate` or `classify_reasoner`

See [Error reference](../reference/errors.md).

## Python services

Python v0.4 is alpha — no report objects, no ontology export. For production pipelines requiring structured reports or export, use Rust crates or wait for v0.9.

Always pass explicit profiles: `Reasoner(path, profile="rdfs")` or `profile="rl"`.

## Observability

Inspect `parse_meta` after load:

- `warnings` — skipped mapping shapes
- `mapped_axiom_count` / `skipped_axiom_count` — see [Protégé vs counts](protege-axiom-counts.md)

Engine reports (`MaterializationReport`, RL report) expose per-rule counts and clashes (RL).

## Related

- [Security](../security.md)
- [Performance](performance.md)
- [Comparison](../comparison.md)
