# Supported OWL Constructs (v0.8.0)

The parser **maps** a subset of OWL TBox axioms into `ontologos-core`. Other constructs are **scanned** for profile detection (`ParseMeta.constructs`) but **skipped** with warnings.

## Mapped to core axioms

| OWL construct | Core axiom |
|---------------|------------|
| `SubClassOf` (named ⊑ named) | `SubClassOf` |
| `SubClassOf` (C ⊑ ∃r.D, named filler) | `SubClassOfExistential` |
| `SubClassOf` (C ⊑ D ⊓ E …, EL operands) | decomposed `SubClassOf` / `SubClassOfExistential` |
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
| `AsymmetricObjectProperty` | `AsymmetricObjectProperty` |
| `EquivalentObjectProperties` (named operands) | `EquivalentObjectProperties` |
| `ClassAssertion` (named individual, named class) | `ClassAssertion` |
| `ObjectPropertyAssertion` (named individuals + property) | `ObjectPropertyAssertion` |
| `SameIndividual` / `DifferentIndividuals` (named) | `SameIndividual` / `DifferentIndividuals` |

Entity declarations (classes, properties, individuals) are registered even when surrounding axioms are skipped.

## Scanned but not mapped

Examples (non-exhaustive):

- Complex class expressions: `ObjectUnionOf`, `ObjectIntersectionOf`, `ObjectComplementOf`, `ObjectAllValuesFrom`, cardinalities, nominals
- `DisjointUnion`, irreflexive/inverse-functional properties
- Negative property assertions, data property assertions
- Data properties and datatypes (declarations may register entities; axioms skipped)
- SWRL rules, annotations (neutral for profile diagnostics)

Skipped axioms increment `parse_meta.skipped_axiom_count` and append to `parse_meta.warnings`.

`owl:imports` declarations are scanned but **not resolved** — imported ontologies are not loaded.

## RDFS materialization scope

| Input in core | Materialized by `ontologos-rdfs` |
|---------------|----------------------------------|
| `SubClassOf` | Transitive closure |
| `SubObjectPropertyOf` | Transitive closure |
| `ObjectPropertyDomain` / `ObjectPropertyRange` | Inherited along `subPropertyOf` |
| `EquivalentClasses` | Stored only; mutual `SubClassOf` via `ontologos-rl` |

## OWL RL saturation scope (v0.4)

`RlEngine::saturate` runs RDFS materialization first, then RL TBox/ABox rules (equivalence, property characteristics, existentials, type/property propagation, `sameAs`, clash detection). See [OWL RL saturation](../getting-started/owl-rl-saturation.md).

**Clash detection (v0.4):** direct disjoint class pairs on an individual's types; `sameAs` / `differentFrom` inconsistency within a `sameAs` cluster. Functional-property duplicate values, asymmetric reverse pairs, and disjointness via subclass closure are not yet reported.

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
