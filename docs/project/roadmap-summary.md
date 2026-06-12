# Roadmap Summary

OntoLogos follows [semantic versioning](https://semver.org/). **Current release:** [v0.5.0](https://github.com/eddiethedean/ontologos/releases/tag/v0.5.0) · **Next milestone:** v0.6 — explanations.

## Shipped (v0.4)

| Area | Status |
|------|--------|
| Core data model, JSON v2, ABox | Available |
| OWL/RDF parser, profile detection | Available |
| RDFS materialization | Available (library + CLI) |
| OWL RL saturation | Available (library + Python) |
| CLI `profile`, `materialize` | Available |
| Python alpha (`profile="rdfs"` / `"rl"`) | Available |

## Next releases

| Version | Theme | Key deliverables |
|---------|-------|------------------|
| **0.5** | OWL EL & query | `ontologos-el`, `ontologos-query`, CLI RL routing, real `classify` |
| **0.6** | Explanations | `ontologos-explain`, CLI `explain` |
| **0.7** | Incremental reasoning | Delta updates across engines |
| **0.9** | Python ecosystem | Full PyPI API |
| **1.0** | Stable release | All 0.x crates, polished CLI |
| **2.0** | Full OWL DL | `ontologos-dl` stable |

## v0.5 documentation plan

See [v0.5 documentation plan](v0.5-docs-plan.md) for the doc updates scheduled with the EL release (CLI RL routing, `classify` rename/alias, EL guides).

## Full roadmap

Exit criteria, crate publish policy, and detailed checklists live in the repository:

**[ROADMAP.md on GitHub](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md)**

Historical ecosystem vision (Ontocode, OntoHub): [PLAN.md on GitHub](https://github.com/eddiethedean/ontologos/blob/main/PLAN.md) — not current scope.
