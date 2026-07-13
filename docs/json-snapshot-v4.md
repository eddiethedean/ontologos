# JSON snapshot v4 (draft on main)

**Status:** **`FORMAT_VERSION = 4`** on workspace **1.1.3** (`main`). **Published v1.0.0** writers still emit **v3** — see [JSON snapshot v3](json-snapshot-v3.md). Readers on `main` accept v2–v4; writers on `main` emit v4.

Legacy formats: [JSON snapshot v2](json-snapshot-v2.md), [JSON snapshot v3](json-snapshot-v3.md).

## Top-level object

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `format_version` | number | yes | Must be `4` for new exports |
| `entities` | array | yes | `{ "iri": string, "kind": EntityKind }` |
| `axioms` | array | yes | Asserted axioms only (inferred/materialized axioms are omitted) |
| `dl` | object | no | OWL DL block (`DlStore`) when present |
| `swrl_rules` | array | no | SWRL rules when present |
| `parse_meta` | object | no | Parser warnings and axiom counts from lenient/OWL load |

## Changes from v3

| Field | v3 | v4 |
|-------|----|----|
| `format_version` | `3` | `4` |
| `parse_meta` | — | Optional `{ warnings, mapped_axiom_count, skipped_axiom_count, logical_axiom_count }` |
| Inferred axioms | Documented optional `inferred: true` | Still omitted on export; `to_json()` logs a warning when inferred axioms exist |

## `parse_meta`

Present when parsing skipped axioms or emitted warnings (for example lenient OWL/RDF load). Omitted when parsing was clean.

```json
"parse_meta": {
  "warnings": ["skipped unsupported axiom ..."],
  "mapped_axiom_count": 12,
  "skipped_axiom_count": 1,
  "logical_axiom_count": 13
}
```

## Compatibility

- v4 writers emit `format_version: 4`.
- v4 readers accept v2, v3, and v4 snapshots.
- Round-trip through JSON preserves asserted axioms, DL, SWRL rules, and `parse_meta`; inferred axioms from RL/RDFS materialization are not exported.

## Security

Untrusted uploads: use `Ontology::from_json_with_limits` — format v1 is rejected. DL and SWRL blocks are validated against default resource limits. See [Security](security.md).

## Related

- [JSON snapshot v3](json-snapshot-v3.md) — previous reference
- [Migration v0.9 → v1.0](migration/v0.9.x-to-v1.0.0.md)
