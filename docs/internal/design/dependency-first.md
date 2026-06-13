# ADR: Dependency-First Architecture

**Status:** Accepted (2026-06-12), amended (2026-06-12)

## Context

OntoLogos previously implemented OWL EL completion, RDFS rules, and OWL RL forward-chaining in-house while treating `whelk-rs` and `reasonable` only as conformance peers. v0.6 briefly delegated EL to **whelk** (git); that blocked crates.io publish and was reverted in favor of the restored in-house EL engine. RL/RDFS still delegate to **reasonable**.

## Decision

Delegate profile-specific reasoning where a maintained crates.io dependency covers the profile; keep EL completion in-tree:

| Concern | Crate |
|---------|-------|
| OWL parsing | `horned-owl` |
| OWL EL classification | **`ontologos-el`** (in-house completion) |
| OWL RL + RDFS materialization | `reasonable` |
| Graph algorithms (taxonomy views, proof graphs) | `petgraph` |
| Parallel batch work | `rayon` |
| Serialization | `serde` |
| Python bindings | `pyo3` |

Public crate names (`ontologos-el`, `ontologos-rl`, `ontologos-rdfs`) remain stable facades. Conversions for RL/RDFS live in `ontologos-bridge`.

## Rules

1. Do not reimplement OWL syntax parsing or RL/RDFS Datalog rules when **reasonable** covers the profile.
2. EL completion stays in `ontologos-el` (ELK-style rules, Pizza golden in CI).
3. Adapter fidelity tests (HermiT Tier A, Pizza golden, Family RL) are release gates.
4. Upstream gaps are tracked as issues/PRs to reasonable, not silently reimplemented.
5. `petgraph` is for views on outputs (taxonomy, proof graphs), not for reimplementing EL completion graphs.

## Known upstream gaps (tracked, not reimplemented)

| Gap | Workaround in OntoLogos |
|-----|-------------------------|
| `reasonable`: no RDFS subProperty/domain/range inheritance (rdfs5–8) | HermiT/RDFS unit tests ignored |
| `reasonable`: no mutual `subPropertyOf` from `equivalentProperty` | HermiT test ignored; file upstream issue |
| `reasonable`: no property-characteristic propagation along `subPropertyOf` | HermiT tests ignored |
| `reasonable`: no existential TBox subsumption between named classes | Use in-house EL; RL tests ignored |
| `reasonable`: domain on subproperty does not type superproperty assertions | HermiT test ignored |
| `reasonable`: no rule-level explanation traces | Proof graphs from asserted axioms only |

## Consequences

- Dual native models: core axioms for EL; oxrdf + reasonable for RL/RDFS.
- EL explanations use in-house `InferenceTrace` on completion rules.
- All library crates except CLI/conformance/py are publishable to crates.io.

## Related

- [Architecture](../../architecture.md)
- [Roadmap summary](../../project/roadmap-summary.md)
- [Rust ecosystem study (repository)](https://github.com/eddiethedean/ontologos/blob/main/docs/internal/research/rust-ecosystem.md)
- [ROADMAP.md (repository)](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md)
