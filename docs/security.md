# Security Considerations

OntoLogos v0.4 handles untrusted input through **JSON deserialization**, **OWL/RDF file parsing**, and **path validation**. This document describes defaults and recommended practices.

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

### IRI validation

Only these schemes are accepted: **`http`**, **`https`**, **`urn`**.

Rejected:

- `javascript:`, `data:`, and other schemes
- Control characters (C0, DEL)
- ASCII whitespace in IRIs
- Relative IRIs (no scheme)

### Format integrity

- **Format v1 is rejected** — positional `iris[]` / entity index binding is unsafe for untrusted input
- **Format v2** keys axioms by IRI string
- Unknown JSON fields on snapshot structs are rejected
- Duplicate entity IRIs are rejected
- Duplicate axioms are deduplicated on load (idempotent)

## File loading (v0.2+)

`ontologos_parser::validate_load_path` canonicalizes paths and rejects traversal outside an optional base directory using **path-component containment** (not string-prefix matching).

- `load_ontology` — no sandbox base (trusted local paths)
- `load_ontology_in(base, path)` — constrain loads to stay under `base` (untrusted uploads)

Both validate the path, enforce [`ParseLimits`](https://docs.rs/ontologos-parser/latest/ontologos_parser/struct.ParseLimits.html), then parse via horned-owl.

### Default parse limits

| Limit | Default | Purpose |
|-------|---------|---------|
| `max_file_bytes` | 64 MiB | Cap ontology file size on disk |
| `max_axioms` | 10,000,000 | Cap stored axioms during mapping |
| `max_entities` | 1,000,000 | Cap registered entities during mapping |

Use `load_ontology_with_limits` for untrusted uploads.

Skipped axioms and parser warnings are recorded in `ParseMeta` but do not fail the load. Review `parse_meta.warnings` when accepting user-supplied ontologies.

## Reporting issues

Report security vulnerabilities privately — see [SECURITY.md](../SECURITY.md) at the repository root (not public GitHub issues).

## Related

- [JSON snapshot v2](json-snapshot-v2.md)
- [Load an OWL file](getting-started/load-owl-file.md)
- [Error reference](reference/errors.md)
