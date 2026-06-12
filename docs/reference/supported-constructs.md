# Supported OWL Constructs (v0.3)

The parser **maps** a subset of OWL TBox axioms into `ontologos-core`. Other constructs are **scanned** for profile detection (`ParseMeta.constructs`) but **skipped** with warnings.

## Mapped to core axioms

| OWL construct | Core axiom |
|---------------|------------|
| `SubClassOf` (named ⊑ named) | `SubClassOf` |
| `SubClassOf` (C ⊑ ∃r.D, named filler) | `SubClassOfExistential` |
| `EquivalentClasses` (named operands) | `EquivalentClasses` |
| `DisjointClasses` (named operands) | `DisjointClasses` |
| `SubObjectPropertyOf` | `SubObjectPropertyOf` |
| `InverseObjectProperties` | `InverseObjectProperties` |
| `ObjectPropertyDomain` (named class) | `ObjectPropertyDomain` |
| `ObjectPropertyRange` (named class) | `ObjectPropertyRange` |
| `TransitiveObjectProperty` | `TransitiveObjectProperty` |
| `SymmetricObjectProperty` | `SymmetricObjectProperty` |
| `ReflexiveObjectProperty` | `ReflexiveObjectProperty` |
| `FunctionalObjectProperty` | `FunctionalObjectProperty` |

Entity declarations (classes, properties, individuals) are registered even when surrounding axioms are skipped.

## Scanned but not mapped (v0.2)

Examples (non-exhaustive):

- Complex class expressions: `ObjectUnionOf`, `ObjectIntersectionOf`, `ObjectComplementOf`, `ObjectAllValuesFrom`, cardinalities, nominals
- `DisjointUnion`, `EquivalentObjectProperties`, asymmetric/irreflexive/inverse-functional properties
- ABox: `ClassAssertion`, `ObjectPropertyAssertion`, individual equality
- Data properties and datatypes (declarations may register entities; axioms skipped)
- SWRL rules, annotations (neutral for profile diagnostics)

Skipped axioms increment `parse_meta.skipped_axiom_count` and append to `parse_meta.warnings`.

`owl:imports` declarations are scanned but **not resolved** — imported ontologies are not loaded.

## RDFS materialization scope (v0.3)

| Input in core | Materialized by `ontologos-rdfs` |
|---------------|----------------------------------|
| `SubClassOf` | Transitive closure |
| `SubObjectPropertyOf` | Transitive closure |
| `ObjectPropertyDomain` / `ObjectPropertyRange` | Inherited along `subPropertyOf` |
| `EquivalentClasses` | Stored only; not expanded to mutual `SubClassOf` (v0.4+ RL) |
| Data properties, ABox, `rdf:type` | Not in scope (parser skips or deferred) |

## Profile detection input

| Field | Contents |
|-------|----------|
| `profile_constructs` | Constructs from **mapped** axioms only — drives detected EL/RL/QL/DL |
| `constructs` | Full source scan — drives **diagnostics** for constructs outside detected profile |

See [Profile detection](../guides/profile-detection.md).

## Related

- [Load an OWL file](../getting-started/load-owl-file.md)
- [Troubleshooting](../guides/troubleshooting.md)
- Mapper implementation: `crates/ontologos-parser/src/map.rs`
