# Profile API Reference

OWL profile detection via [`ontologos-profile`](https://docs.rs/ontologos-profile/1.1.3).

Guide: [Profile detection](../guides/profile-detection.md).

## Primary functions

| Function | Returns | Description |
|----------|---------|-------------|
| `detect_profile(&Ontology)` | `ProfileReport` | Detect EL/RL/RDFS/QL/DL from mapped constructs |
| `classify_hybrid(&Ontology)` | `HybridReport` | Multi-module DL routing diagnostics |

## `ProfileReport`

| Field | Description |
|-------|-------------|
| `detected` | Primary profile (`El`, `Rl`, `Dl`, …) |
| `diagnostics` | Constructs that rule out simpler profiles |
| `constructs` | Mapped construct set |

Requires `parse_meta` from file load (or synthetic metadata for builder-only ontologies).

## CLI equivalent

```bash
ontologos profile ontology.owl
```

## Errors

Transparent `ontologos_core::Error` for invalid ontology state.

## Example

```rust
use ontologos_parser::load_ontology;
use ontologos_profile::detect_profile;

let ontology = load_ontology(path)?;
let report = detect_profile(&ontology)?;
println!("{:?}", report.detected);
```

## docs.rs

Full API: [docs.rs/ontologos-profile/1.1.3](https://docs.rs/ontologos-profile/1.1.3)

## Related

- [Profile stability matrix](../guides/profile-stability.md)
- [Core `Profile` enum](core.md#profile)
