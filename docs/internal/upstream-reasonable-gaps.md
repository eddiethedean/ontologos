# Upstream gaps: `reasonable` OWL RL adapter

OntoLogos delegates OWL RL and RDFS materialization to the [`reasonable`](https://crates.io/crates/reasonable) crate via `ontologos-bridge`. The following semantic gaps are confirmed by regression tests in `crates/ontologos-rl/tests/regression_bugs.rs` and mirrored in `ontologos-conformance/tests/hermit_rl.rs`.

**Bridge fallbacks (v0.9+):** `ontologos-bridge::apply_reasonable_fallbacks` post-processes after reasonable materialization for:
- `EqPropSub`: mutual `subPropertyOf` from `equivalentProperty`
- `CharPropagate`: property characteristics along `subPropertyOf` (functional/asymmetric/irreflexive down, reflexive up)
- Transitive `subPropertyOf` (RDFS 5)
- Domain/range inheritance along `subPropertyOf` (prp-dom / prp-rng)
- `cls-svf1/2`: existential TBox subsumption between named classes (`∃P.C` / `∃Q.D` with `P ⊑ Q` and `C ⊑ D`)

Track remaining fixes upstream; un-ignore OntoLogos tests when `reasonable` releases include the behavior natively.

| Test | W3C rule / issue | Description | Bridge fallback |
|------|------------------|-------------|-----------------|
| `domain_on_subproperty_types_superproperty_assertion` | prp-dom2 | Domain on subproperty `Q` should type assertions using superproperty `P` | **Yes** (active) |
| `domain_on_transitive_subproperty_types_superproperty_assertion` | prp-dom | Domain not inherited along `subPropertyOf` chains | **Yes** (active) |
| `range_on_subproperty_types_superproperty_assertion_object` | prp-rng2 | Range on subproperty should type object of superproperty assertion | **Yes** (active) |
| `functional_property_characteristic_propagates_to_subproperty` | CharPropagate | Functional propagates to subproperties | **Yes** (active) |
| `asymmetric_property_characteristic_propagates_to_subproperty` | CharPropagate | Asymmetric propagates to subproperties | **Yes** (active) |
| `equivalent_properties_mutual_subproperty` | EqPropSub | Equivalent properties → mutual subPropertyOf | **Yes** (active) |
| `existential_propagates_along_subclass_of` | scm-spo1 | `∃r.C` propagates to subclass | **Yes** (reasonable + index; active) |
| `existential_subsumption_with_filler_subclass` | cls-svf2 | Filler subsumption enables existential class subsumption | **Yes** (active) |
| HermiT `subsumption2/3` | cls-svf1 | Property subsumption/equivalence + `EquivalentClasses` existentials | **Yes** (active) |
| `same_as_different_from_clash_deduped_across_iterations` | — | Clash diagnostics differ from legacy RL dedup semantics | No |

## Remaining upstream gaps (no bridge fallback)

| Gap | Tests |
|-----|-------|
| Clash dedup semantics (sameAs/differentFrom) | `same_as_different_from_clash_deduped_across_iterations` |
| Rule-level explanation traces | `with_traces()` no-op; `inferred_by_rule` empty |
| Parallelism ignored | `parallel_smoke.rs` |

## Catalog promotion (Wave 1)

`generate_catalog.py` now extracts `property_characteristics`, `property_subsumptions`, and consistency assertions. Active RL/RDFS `axiom` cases include `testIsReflexiveObject`, `testSubRolesChain`, and datatype consistency smokes.

## Preferred resolution

1. File issues or PRs against the `reasonable` repository for each remaining row.
2. Bump the workspace `reasonable` version in `Cargo.toml` when fixes land.
3. Run `cargo test -p ontologos-rl --test regression_bugs -- --ignored` and un-ignore passing tests.
4. Once reasonable implements a rule natively, remove the corresponding bridge fallback to avoid double application.

## RDFS/RL explanations

`ontologos-rl` and `ontologos-rdfs` document `with_traces()` as a no-op until `reasonable` exposes rule-firing diagnostics. Do not claim explain support for RL/RDFS profiles until traces are populated.
