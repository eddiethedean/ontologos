# SWRL API Reference

DLSafe SWRL + DL via [`ontologos-swrl`](https://docs.rs/ontologos-swrl/1.1.2).

Tutorial: [SWRL quick start](../getting-started/swrl.md).

## Primary entry points

SWRL is routed through **`ontologos-facade`** with `Profile::Swrl` — prefer the facade for CLI/Python parity.

| Function / type | Description |
|-----------------|-------------|
| `ontologos_facade::classify` with `Profile::Swrl` | Full SWRL + DL pipeline |
| `ontologos_facade::check_consistency` | Consistency with rule grounding |
| `ontologos_core::SwrlRule` | In-memory rule representation |

Direct crate use is for engine contributors; integrators should use the facade.

## Python

```python
Reasoner(path="rules.owl", profile="swrl", budget_secs=30).classify()
```

## Constraints

- **DLSafe** rules only — unsafe rules skipped at parse time
- Requires DL-capable ontology shapes for full grounding
- Set wall-clock `budget_secs` for production

## Errors

Wrapped as `ontologos_facade::Error::Swrl` or `Error::Dl` depending on failure phase.

## docs.rs

Full API: [docs.rs/ontologos-swrl/1.1.2](https://docs.rs/ontologos-swrl/1.1.2)

## Related

- [DL API](dl.md)
- [Supported constructs](supported-constructs.md)
- [Profile stability](../guides/profile-stability.md)
