# JSON Snapshot Format v2

OntoLogos serializes ontologies to JSON for storage and interchange. **Format version 2 only** for untrusted input; v1 is rejected.

## Top-level schema

```json
{
  "format_version": 2,
  "entities": [
    { "iri": "http://example.org/Pizza", "kind": "Class" }
  ],
  "axioms": [
    {
      "SubClassOf": {
        "subclass": "http://example.org/Pizza",
        "superclass": "http://example.org/Food"
      }
    }
  ]
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `format_version` | `u32` | yes | Must be `2` |
| `entities` | array | yes | Entity declarations |
| `axioms` | array | yes | Axiom declarations |

Unknown top-level fields are rejected (`deny_unknown_fields`).

## Entity record

```json
{ "iri": "http://example.org/Pizza", "kind": "Class" }
```

| Field | Description |
|-------|-------------|
| `iri` | Absolute IRI string (`http`, `https`, or `urn`) |
| `kind` | One of: `Class`, `Individual`, `ObjectProperty`, `DataProperty`, `AnnotationProperty` |

Duplicate `iri` values in `entities` are rejected.

## Axiom records

Axioms use IRI strings (not numeric entity IDs). Supported variants:

| Variant | JSON shape |
|---------|------------|
| `SubClassOf` | `{ "subclass": "...", "superclass": "..." }` |
| `EquivalentClasses` | `[ "...", "..." ]` |
| `DisjointClasses` | `[ "...", "..." ]` |
| `ObjectPropertyDomain` | `{ "property": "...", "domain": "..." }` |
| `ObjectPropertyRange` | `{ "property": "...", "range": "..." }` |
| `SubObjectPropertyOf` | `{ "sub_property": "...", "super_property": "..." }` |
| `InverseObjectProperties` | `{ "left": "...", "right": "..." }` |
| `TransitiveObjectProperty` | `"..."` (property IRI string) |
| `SubClassOfExistential` | `{ "subclass": "...", "property": "...", "filler": "..." }` *(v0.2)* |
| `SymmetricObjectProperty` | `"..."` *(v0.2)* |
| `ReflexiveObjectProperty` | `"..."` *(v0.2)* |
| `FunctionalObjectProperty` | `"..."` |
| `AsymmetricObjectProperty` | `"..."` *(v0.4)* |
| `EquivalentObjectProperties` | `[ "...", "..." ]` *(v0.4)* |
| `ClassAssertion` | `{ "individual": "...", "class": "..." }` *(v0.4)* |
| `ObjectPropertyAssertion` | `{ "subject": "...", "property": "...", "object": "..." }` *(v0.4)* |
| `SameIndividual` | `[ "...", "..." ]` *(v0.4)* |
| `DifferentIndividuals` | `[ "...", "..." ]` *(v0.4)* |

Every IRI referenced in an axiom must match an entity declared in `entities`.

### v0.4 ABox examples

```json
{
  "ClassAssertion": {
    "individual": "http://example.org/alice",
    "class": "http://example.org/Person"
  }
}
```

```json
{
  "ObjectPropertyAssertion": {
    "subject": "http://example.org/alice",
    "property": "http://example.org/knows",
    "object": "http://example.org/bob"
  }
}
```

```json
{ "SameIndividual": ["http://example.org/alice", "http://example.org/alice-copy"] }
```

```json
{ "DifferentIndividuals": ["http://example.org/alice", "http://example.org/bob"] }
```

```json
{ "AsymmetricObjectProperty": "http://example.org/hasParent" }
```

```json
{
  "EquivalentObjectProperties": [
    "http://example.org/hasParent",
    "http://example.org/isParentOf"
  ]
}
```

## Full example

See [pizza_minimal.json](../crates/ontologos-core/tests/fixtures/pizza_minimal.json).

## Rust API

```rust
use ontologos_core::{Limits, Ontology};

let json = ontology.to_json()?;
let restored = Ontology::from_json(&json)?;

// Custom limits for untrusted input:
let strict = Limits {
    max_json_bytes: 1_048_576,
    ..Limits::default()
};
let limited = Ontology::from_json_with_limits(&json, strict)?;
```

## Default deserialization limits

| Limit | Default |
|-------|---------|
| `max_json_bytes` | 16 MiB |
| `max_entities` | 1,000,000 |
| `max_axioms` | 10,000,000 |
| `max_iri_len` | 8,192 bytes |
| `max_class_operands` | 10,000 |

See [security.md](security.md) for untrusted input guidance.

## Migration from v1

Format v1 used a separate `iris[]` table and numeric entity indices in axioms. **v1 is not accepted** by `Ontology::from_json` (returns `Error::Serialization`).

To migrate:

1. Re-export from Rust: `ontology.to_json()` on a v0.1-built ontology produces v2.
2. Or hand-convert: replace `iris` + `iri_index` with inline `iri` strings on entities; replace numeric axiom refs with IRI strings.

## Related

- [Error reference](reference/errors.md)
- [SPEC.md](../SPEC.md) — core data model
