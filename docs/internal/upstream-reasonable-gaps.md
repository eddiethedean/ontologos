# Upstream gaps: `reasonable` OWL RL adapter

OntoLogos delegates OWL RL and RDFS materialization to the [`reasonable`](https://crates.io/crates/reasonable) crate via `ontologos-bridge`. The following semantic gaps are confirmed by ignored regression tests in `crates/ontologos-rl/tests/regression_bugs.rs` and mirrored in `ontologos-conformance/tests/hermit_rl.rs`.

Track fixes upstream; un-ignore OntoLogos tests when `reasonable` releases include the behavior.

| Test | W3C rule / issue | Description |
|------|------------------|-------------|
| `domain_on_subproperty_types_superproperty_assertion` | prp-dom2 | Domain on subproperty `Q` should type assertions using superproperty `P` |
| `domain_inherited_along_subproperty_chain` | prp-dom | Domain not inherited along `subPropertyOf` chains |
| `range_on_subproperty_types_superproperty_assertion` | prp-rng2 | Range on subproperty should type object of superproperty assertion |
| `existential_tbox_subclass_named_classes` | — | `∃r.C ⊑ D` not materialized as named `SubClassOf` |
| `existential_tbox_subclass_second_case` | — | Second existential TBox materialization variant |
| `rl_clash_dedup_semantics` | — | Clash diagnostics differ from legacy RL dedup semantics |

## Preferred resolution

1. File issues or PRs against the `reasonable` repository for each row.
2. Bump the workspace `reasonable` version in `Cargo.toml` when fixes land.
3. Run `cargo test -p ontologos-rl --test regression_bugs -- --ignored` and un-ignore passing tests.

## Fallback (if upstream is slow)

Post-process inferred triples in `crates/ontologos-bridge/src/reasonable_session.rs` for high-value rules (domain/range inheritance along `subPropertyOf`). Only pursue if upstream rejects the fix.

## RDFS/RL explanations

`ontologos-rl` and `ontologos-rdfs` document `with_traces()` as a no-op until `reasonable` exposes rule-firing diagnostics. Do not claim explain support for RL/RDFS profiles until traces are populated.
