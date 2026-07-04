# JSON snapshot v3

**Status:** Shipped in workspace **1.1.0** (`FORMAT_VERSION = 3` in `ontologos-core`). Readers accept v2 and v3; writers emit v3.

Legacy v2 format: [JSON snapshot v2](json-snapshot-v2.md).

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
- v2-only snapshots remain readable.

## Security

Untrusted uploads: use `Ontology::from_json_with_limits` — format v1 is rejected. See [Security](security.md).

## Related

- [JSON snapshot v2](json-snapshot-v2.md) — legacy reference
- [Migration v0.9 → v1.0](migration/v0.9.x-to-v1.0.0.md)
