# Security Considerations

OntoLogos v1.0.0 handles untrusted input through **JSON deserialization**, **OWL/RDF file parsing**, and **path validation**. This document describes defaults and recommended practices.

## JSON snapshots

Use `Ontology::from_json` for trusted snapshots. For **untrusted** input (user uploads, network payloads), prefer:

```rust
use ontologos_core::{Limits, Ontology};

let limits = Limits {
    max_json_bytes: 1_048_576, // 1 MiB
    ..Limits::default()
};
let ontology = Ontology::from_json_with_limits(json, limits)?;
```

### Default limits

| Limit | Default | Purpose |
|-------|---------|---------|
| `max_json_bytes` | 16 MiB | Prevent memory exhaustion |
| `max_entities` | 1,000,000 | Cap entity array size |
| `max_axioms` | 10,000,000 | Cap axiom array size |
| `max_iri_len` | 8,192 | Cap per-IRI string length |
| `max_class_operands` | 10,000 | Cap equivalent/disjoint operands |
| `max_literal_bytes` | 1 MiB | Cap lexical form length for data literals |

### IRI validation

JSON snapshots accept only **`http`**, **`https`**, and **`urn`** schemes via `validate_snapshot_iri`.

OWL file parsing (via `ontologos-parser`) also accepts **`file`** and **`internal`** for legacy corpora and internal DL surrogates.

Rejected everywhere:

- `javascript:`, `data:`, and other schemes
- Control characters (C0, DEL)
- ASCII whitespace in IRIs
- Relative IRIs (no scheme)

Literal `datatype` IRIs in JSON snapshots are validated the same way as entity IRIs.

### Format integrity

- **Format v1 is rejected** — positional `iris[]` / entity index binding is unsafe for untrusted input
- **Format v3** (writers on workspace 1.0.0) and **v2** (legacy) key axioms by IRI string; readers accept both
- Unknown JSON fields on snapshot structs are rejected
- Duplicate entity IRIs are rejected
- Duplicate axioms are deduplicated on load (idempotent)

## File loading (v0.2+)

`ontologos_parser::validate_load_path` canonicalizes paths and rejects traversal outside an optional base directory using **path-component containment** (not string-prefix matching).

- `load_ontology` — no sandbox base (trusted local paths); **strict by default**; merges local `owl:imports` for RDF/XML
- `load_ontology_lenient` — same as `load_ontology` but allows skipped axioms with warnings
- `load_ontology_in(base, path)` — constrain loads to stay under `base` (untrusted uploads)

Loads validate the path, enforce [`ParseLimits`](https://docs.rs/ontologos-parser/0.9.0/ontologos_parser/struct.ParseLimits.html), run a lightweight axiom/component pre-scan, then parse via horned-owl. Post-load lightweight validation runs on every successful load; expensive blank-node graph checks run when `strict` is true. Malformed RDF/XML that triggers horned-owl internal panics is converted to `Error::Parse`. Sandboxed loads open the file once with `O_NOFOLLOW` (Unix) and sniff plus parse from the same file descriptor so a symlink swap between validation and read cannot escape the base directory.

### Parser concurrency (server embedders)

`ontologos-parser` serializes horned-owl reader entry points with a **process-wide mutex** (`HORNED_OWL_READ_LOCK`). Concurrent OWL file loads from multiple threads **block** on this lock; horned-owl may panic or corrupt internal state if invoked without serialization.

**Production guidance:**

- Treat OWL parsing as **single-threaded per process** (one load at a time), or isolate loads in **separate worker processes**
- Do not assume `load_ontology_in` sandboxing alone makes parallel loads safe
- JSON snapshot deserialization (`Ontology::from_json_with_limits`) does not use horned-owl and is not subject to this mutex

### Reasoning merge limits (v0.9.0+)

RL/RDFS saturation merges inferred axioms via `ontologos-bridge::MergeLimits` (default `max_axioms: 10_000_000`). Configure with `RdfsEngine::with_merge_limits` / `RlEngine::with_merge_limits` for untrusted workloads.

### Default parse limits

| Limit | Default | Purpose |
|-------|---------|---------|
| `max_file_bytes` | 64 MiB | Cap ontology file size on disk |
| `max_preprocess_bytes` | 512 MiB (8× file) | Cumulative RDF/XML preprocessing allocation |
| `max_harvested_assertions` | 100,000 | Cap RDF/XML supplement harvest per load |
| `max_axioms` | 10,000,000 | Cap stored axioms during mapping |
| `max_entities` | 1,000,000 | Cap registered entities during mapping |
| `merge_imports` | **false** in `ParseLimits::default()` | Opt in explicitly; trusted `load_ontology*` helpers set `true` |
| `strict` | **true** | Fail on skipped axioms / incompatible declarations; use `load_ontology_lenient` to opt out |

Use `load_ontology_with_limits` or `load_ontology_in` for untrusted uploads. For user-supplied paths, prefer **`load_ontology_in`** with a sandbox base and **`merge_imports: false`** unless you trust sibling import files. Skipped axioms and parser warnings are recorded in `ParseMeta`; with `strict: true` (default) they fail the load instead.

## Conformance harness environment (do not set in production)

DL consistency and classification paths may enable **WG corpus shortcuts** when `ONTOLOGOS_CONFORMANCE=1` (CI and local conformance runs). These shortcuts speed HermiT catalog cases and are **not** used for production reasoning when the variable is unset.

**Do not set** in production or customer-facing services:

| Variable | Purpose |
|----------|---------|
| `ONTOLOGOS_CONFORMANCE` | WG wine/import consistency shortcuts in DL |
| `ONTOLOGOS_STRICT_TAXONOMY` | Tier C strict taxonomy comparison (harness only) |
| `ONTOLOGOS_CI_PROMOTED_ONLY` | Subset conformance runs in CI |

Validate DL results on **your own corpus** with `check_consistency` and `budget_secs` — see [Production integration](guides/production-integration.md#owl-dl-in-production).

## Reporting issues

Report security vulnerabilities privately — see [Security policy](project/security-policy.md) (not public GitHub issues).

## Related

- [JSON snapshot v3](json-snapshot-v3.md) · [v2 legacy](json-snapshot-v2.md)
- [Load an OWL file](getting-started/load-owl-file.md)
- [Error reference](reference/errors.md)
- [Production integration](guides/production-integration.md)
