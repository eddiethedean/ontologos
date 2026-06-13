# Upstream gaps: `reasonable` OWL RL adapter

OntoLogos delegates OWL RL and RDFS materialization to the [`reasonable`](https://crates.io/crates/reasonable) crate via `ontologos-bridge`. The following semantic gaps are confirmed by regression tests in `crates/ontologos-rl/tests/regression_bugs.rs` and mirrored in `ontologos-conformance/tests/hermit_rl.rs`.

**Bridge fallbacks (v0.9+):** `ontologos-bridge::apply_reasonable_fallbacks` post-processes after reasonable materialization for:
- Transitive `subPropertyOf` (RDFS 5)
- Domain/range inheritance along `subPropertyOf` (prp-dom / prp-rng)

Track remaining fixes upstream; un-ignore OntoLogos tests when `reasonable` releases include the behavior natively.

| Test | W3C rule / issue | Description | Bridge fallback |
|------|------------------|-------------|-----------------|
| `domain_on_subproperty_types_superproperty_assertion` | prp-dom2 | Domain on subproperty `Q` should type assertions using superproperty `P` | **Yes** |
| `domain_on_transitive_subproperty_types_superproperty_assertion` | prp-dom | Domain not inherited along `subPropertyOf` chains | **Yes** |
| `range_on_subproperty_types_superproperty_assertion_object` | prp-rng2 | Range on subproperty should type object of superproperty assertion | **Yes** |
| `existential_propagates_along_subclass_of` | scm-spo1 | `∃r.C ⊑ D` not materialized as named `SubClassOf` | No |
| `existential_subsumption_with_filler_subclass` | scm-spo1 | Second existential TBox materialization variant | No |
| `same_as_different_from_clash_deduped_across_iterations` | — | Clash diagnostics differ from legacy RL dedup semantics | No |

## Remaining upstream gaps (no bridge fallback)

| Gap | Tests |
|-----|-------|
| Existential TBox not materialized as named `SubClassOf` | `existential_propagates_along_subclass_of`, `existential_subsumption_with_filler_subclass`, HermiT `hermit_rl` ports |
| Clash dedup semantics (sameAs/differentFrom) | `same_as_different_from_clash_deduped_across_iterations` |
| Property-characteristic propagation along `subPropertyOf` | `hermit_rl.rs` (2 tests) |
| `equivalentProperty` → mutual `subPropertyOf` | `hermit_rl.rs` (1 test) |
| Rule-level explanation traces | `with_traces()` no-op; `inferred_by_rule` empty |
| Parallelism ignored | `parallel_smoke.rs` |

## Preferred resolution

1. File issues or PRs against the `reasonable` repository for each remaining row.
2. Bump the workspace `reasonable` version in `Cargo.toml` when fixes land.
3. Run `cargo test -p ontologos-rl --test regression_bugs -- --ignored` and un-ignore passing tests.
4. Once reasonable implements a rule natively, remove the corresponding bridge fallback to avoid double application.

## RDFS/RL explanations

`ontologos-rl` and `ontologos-rdfs` document `with_traces()` as a no-op until `reasonable` exposes rule-firing diagnostics. Do not claim explain support for RL/RDFS profiles until traces are populated.
