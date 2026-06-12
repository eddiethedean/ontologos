# OWL RL Rules Reference

Rule names emitted in `MaterializationReport::inferred_by_rule` after `RlEngine::saturate`. JSON serialization uses snake_case (via `RlRule::as_str()`).

RDFS rules (`subclass_trans`, `subprop_trans`, `dom_inherit`, `rng_inherit`) run in the RDFS pass before RL; their counts appear in `report.rdfs_inferred`, not in `inferred_by_rule`.

## TBox rules

| Rule key | Meaning |
|----------|---------|
| `eq_class_sub` | `EquivalentClasses` expanded to mutual `SubClassOf` |
| `eq_prop_sub` | `EquivalentObjectProperties` expanded to mutual `SubObjectPropertyOf` |
| `char_propagate` | Property characteristics (functional, transitive, symmetric, etc.) propagate along `subPropertyOf` |
| `existential_sub_prop` | Existential restriction propagates along `subPropertyOf`, `equivalentProperties`, `equivalentClasses`, and downward along `subClassOf` (scm-spo1) |
| `existential_subsumption` | Class with `∃P.D` subsumed by class with `∃Q.D` when `P ⊑ Q` (asserted existentials only) |

## ABox rules

| Rule key | Meaning |
|----------|---------|
| `type_domain` | Individual typed by property domain from property assertion |
| `type_range` | Individual typed by property range from property assertion |
| `type_subclass` | Class assertion propagates along `subClassOf` |
| `type_equivalent` | Class assertion propagates along class equivalence |
| `prop_sub` | Property assertion propagates along `subPropertyOf` |
| `prop_inverse` | Property assertion via inverse property |
| `prop_symmetric` | Property assertion via symmetric property |
| `prop_transitive` | Transitive closure on property assertions |
| `same_as_class` | `sameAs` propagates class assertions |
| `same_as_prop` | `sameAs` propagates property assertions |
| `prop_reflexive` | Reflexive property adds self-assertion |

## Report fields

```rust
pub struct MaterializationReport {
    pub initial_axiom_count: usize,
    pub final_axiom_count: usize,
    pub rdfs_inferred: usize,
    pub inferred_by_rule: BTreeMap<RlRule, usize>,
    pub traces: Vec<InferenceRecord>,  // when with_traces(true)
    pub clashes: Vec<String>,
}
```

`inferred_total()` = `final_axiom_count - initial_axiom_count` (RDFS + RL combined).

Enable per-inference traces for future explanation work:

```rust
let report = RlEngine::try_new(1)?
    .with_traces(true)
    .saturate(&mut ontology)?;
```

## Clash detection (v0.4)

Reported in `clashes` when found:

- Individual with types from a declared disjoint class pair
- `sameAs` cluster containing conflicting `differentFrom` assertions

Not yet reported: functional-property duplicate values, asymmetric reverse pairs, disjointness via subclass closure.

See [Supported constructs](supported-constructs.md).

## Related

- [OWL RL saturation guide](../getting-started/owl-rl-saturation.md)
- [docs.rs/ontologos-rl](https://docs.rs/ontologos-rl/0.5.0)
