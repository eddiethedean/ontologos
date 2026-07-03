# RL API Reference

OWL RL saturation and RDFS materialization via [`ontologos-rl`](https://docs.rs/ontologos-rl/1.0.0).

Tutorials: [RDFS materialization](../getting-started/rdfs-materialization.md) · [OWL RL saturation](../getting-started/owl-rl-saturation.md).

## Modules

| Module | Engine | Use |
|--------|--------|-----|
| `ontologos_rl::rdfs` | `RdfsEngine` | RDFS TBox materialization |
| `ontologos_rl` | `RlEngine` | OWL RL saturation (RDFS first) |

## Primary types

| Type | Role |
|------|------|
| `RdfsEngine` | RDFS materialization |
| `RlEngine` | RL saturation with optional parallelism |
| `MaterializationReport` | Initial/final axiom counts, inferred totals |

## Functions

| Function | Description |
|----------|-------------|
| `RdfsEngine::new().materialize(&mut Ontology)` | RDFS closure |
| `RlEngine::new(n)?.saturate(&mut Ontology)` | RL to fixed point (`n` = worker threads, 1..=64) |
| `classify_reasoner(&mut Reasoner)` | RL via reasoner session |
| `ontologos_rl::rdfs::classify_reasoner` | RDFS via reasoner session |

## Errors (`ontologos_rl::Error`)

| Variant | Meaning |
|---------|---------|
| `Reasonable` | Upstream reasonable adapter error |
| `Core` | `ontologos_core::Error` |
| `WrongProfile` | Profile mismatch |

## Example

```rust
use ontologos_rl::RlEngine;

let report = RlEngine::new(1)?.saturate(&mut ontology)?;
println!("inferred {}", report.inferred_total());
```

## Upstream limits

RDFS gaps in reasonable: [Reasonable adapter limits](reasonable-limits.md). RL rules: [RL rules](rl-rules.md).

## docs.rs

Full API: [docs.rs/ontologos-rl/1.0.0](https://docs.rs/ontologos-rl/1.0.0)

## Related

- [Facade API](facade.md) — `Profile::Rl` / `Profile::Rdfs`
