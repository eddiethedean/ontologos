# Supported OWL Constructs (v1.0.0)

The parser maps OWL into two layers:

1. **`ontologos-core` flat axioms** — EL/RL-friendly TBox and ABox (see table below).
2. **`DlStore` via `map_dl.rs`** — full OWL 2 DL class expressions (unions, complements, nominals, cardinalities, data ranges, etc.) for `--profile dl` / `ontologos-dl`.

Profile detection uses both mapped core axioms and `ParseMeta` scans. `axiom_count()` counts **mapped core axioms**, not Protégé's full logical axiom total nor every `DlAxiom`.

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

## Scanned but not mapped to core (DL path may still apply)

Complex class expressions and data axioms are often stored in **`DlStore`** when using `ontologos-parser` + `--profile dl`, even when they do not appear as flat `ontologos-core` axioms:

- `ObjectUnionOf`, `ObjectIntersectionOf`, `ObjectComplementOf`, `ObjectOneOf`, cardinalities, `HasValue`, `HasSelf`
- Data properties, datatypes, and data restrictions (datatype consistency via `ontologos-dl`)
- `DisjointUnion`, irreflexive/inverse-functional properties (partial support)

Still skipped or partial for **core-only** workflows (EL/RL without DL):

- Negative property assertions
- SWRL rules (separate `map_swrl.rs` path)
- Annotations (neutral for profile diagnostics)

Skipped flat axioms increment `parse_meta.skipped_axiom_count` and append to `parse_meta.warnings`.

`owl:imports` for **RDF/XML** loads: local import documents are merged by default (`ParseLimits::merge_imports`, default **true**). Disable with `ParseLimits { merge_imports: false, .. }` for single-document loads only. **Turtle** and **OWL Functional** do not merge imports. **Remote import IRIs are never fetched.**

See [OWL imports reference](owl-imports.md).

## RDFS materialization scope

`ontologos-rdfs` delegates to **reasonable**. The table below describes intended RDFS semantics; gaps are tracked in [Reasonable adapter limits](reasonable-limits.md) and the [dependency-first ADR](../internal/design/dependency-first.md).

| Input in core | Intended materialization | Implemented via reasonable |
|---------------|--------------------------|----------------------------|
| `SubClassOf` | Transitive closure | Yes |
| `SubObjectPropertyOf` | Transitive closure | **No** (upstream gap) |
| `ObjectPropertyDomain` / `ObjectPropertyRange` | Inherited along `subPropertyOf` | **No** (upstream gap) |
| `EquivalentClasses` | Stored only; mutual `SubClassOf` via `ontologos-rl` | Partial |

Use `inferred_total()` and axiom counts for saturation metrics. `MaterializationReport::inferred_by_rule` may be empty until reasonable exposes rule-level diagnostics.

## OWL RL saturation scope

`RlEngine::saturate` runs RDFS materialization first, then RL TBox/ABox rules (equivalence, property characteristics, existentials, type/property propagation, `sameAs`, clash detection). Some RL rules depend on RDFS gaps above — see [Reasonable adapter limits](reasonable-limits.md). See [OWL RL saturation](../getting-started/owl-rl-saturation.md).

**Clash detection:** direct disjoint class pairs on an individual's types; `sameAs` / `differentFrom` inconsistency within a `sameAs` cluster. Functional-property duplicate values, asymmetric reverse pairs, and disjointness via subclass closure are not yet reported.

## Profile detection input

| Field | Contents |
|-------|----------|
| `profile_constructs` | Constructs from **mapped** axioms only — drives detected EL/RL/QL/DL |
| `constructs` | Full source scan — drives **diagnostics** for constructs outside detected profile |

See [Profile detection](../guides/profile-detection.md).

## Related

- [Load an OWL file](../getting-started/load-owl-file.md)
- [Reasonable adapter limits](reasonable-limits.md)
- [Troubleshooting](../guides/troubleshooting.md)
- Mapper implementation: `crates/ontologos-parser/src/map.rs` (core), `crates/ontologos-parser/src/map_dl.rs` (DL)
