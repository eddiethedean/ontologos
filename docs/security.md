# Security Considerations

OntoLogos v0.1 handles untrusted input primarily through **JSON deserialization** and **path validation** (for future file loading). This document describes defaults and recommended practices.

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

`ontologos_parser::validate_load_path` canonicalizes paths and rejects traversal outside an optional base directory. `load_ontology` validates the path before calling `Ontology::from_file`.

Until v0.2, file parsing returns `Error::ParseNotAvailable`; path validation still runs.

## Reporting issues

Report security vulnerabilities privately to the maintainer via GitHub security advisories or email listed in crate metadata.

## Related

- [JSON snapshot v2](json-snapshot-v2.md)
- [Error reference](reference/errors.md)
