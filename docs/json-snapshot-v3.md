# JSON snapshot v3 (planned)

**Status:** Specification for v1.6 ABox milestone. Reader support lands with `FORMAT_VERSION = 3` in `ontologos-core`.

## Changes from v2

| Field | v2 | v3 |
|-------|----|----|
| `format_version` | `2` | `3` |
| `DataPropertyAssertion` | — | Individual + data property + typed literal |
| `NegativeObjectPropertyAssertion` | — | Subject, property, object |
| `NegativeDataPropertyAssertion` | — | Individual, data property, literal |
| Inferred axiom metadata | Optional | Optional `inferred: true` on snapshot axioms |

## Compatibility

- v3 writers emit `format_version: 3`.
- v3 readers accept v2 snapshots (upgrade on load).
- v2 readers continue to work for v2-only snapshots.

## Round-trip tests

Planned in `ontologos-core/src/serialize.rs` when ABox axioms move from `DlAxiom` to core `Axiom`.
