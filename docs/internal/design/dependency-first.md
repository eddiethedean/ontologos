# ADR: Dependency-First Architecture

**Status:** Accepted (2026-06-12)

## Context

OntoLogos previously implemented OWL EL completion, RDFS rules, and OWL RL forward-chaining in-house while treating `whelk-rs` and `reasonable` only as conformance peers. Those crates are actively maintained and outperform reimplemented rule engines.

## Decision

Delegate profile-specific reasoning to ecosystem crates and keep OntoLogos as an orchestration layer:

| Concern | Crate |
|---------|-------|
| OWL parsing | `horned-owl` |
| OWL EL classification | `whelk` (git) |
| OWL RL + RDFS materialization | `reasonable` |
| Graph algorithms (taxonomy views, proof graphs) | `petgraph` |
| Parallel batch work | `rayon` |
| Serialization | `serde` |
| Python bindings | `pyo3` |

Public crate names (`ontologos-el`, `ontologos-rl`, `ontologos-rdfs`) remain stable facades. Conversions live in `ontologos-bridge`.

## Rules

1. Do not reimplement OWL syntax parsing, EL completion rules, or RL Datalog rules when a maintained dependency covers the profile.
2. Adapter fidelity tests (HermiT Tier A, Pizza golden, Family RL) are release gates.
3. Upstream gaps are tracked as issues/PRs to whelk/reasonable, not silently reimplemented.
4. `petgraph` is for views on outputs (taxonomy, proof graphs), not for reimplementing whelk's completion graph.

## Known upstream gaps (tracked, not reimplemented)

| Gap | Workaround in OntoLogos |
|-----|-------------------------|
| `reasonable`: no RDFS subProperty/domain/range inheritance (rdfs5–8) | HermiT/RDFS unit tests ignored |
| `reasonable`: no mutual `subPropertyOf` from `equivalentProperty` | HermiT test ignored; file upstream issue |
| `reasonable`: no property-characteristic propagation along `subPropertyOf` | HermiT tests ignored |
| `reasonable`: no existential TBox subsumption between named classes | Use `whelk` for EL; RL tests ignored |
| `reasonable`: domain on subproperty does not type superproperty assertions | HermiT test ignored |
| `whelk`: no rule-level explanation traces | EL taxonomy only; empty `InferenceTrace` |
| `reasonable`: no rule-level explanation traces | Proof graphs from asserted axioms only |

## Consequences

- Dual native models: horned-owl + whelk for EL; oxrdf + reasonable for RL/RDFS.
- RL explanations remain EL-first until reasonable exposes rule traces.
- `whelk` is pinned as a git dependency until published to crates.io.
- `ontologos-bridge`, `ontologos-el`, `ontologos-rdfs`, `ontologos-rl`, and `ontologos-explain` are workspace-only on crates.io until then.

## Related

- [Architecture](../../architecture.md)
- [Roadmap summary](../../project/roadmap-summary.md)
- [Rust ecosystem study (repository)](https://github.com/eddiethedean/ontologos/blob/main/docs/internal/research/rust-ecosystem.md)
- [ROADMAP.md (repository)](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md)
