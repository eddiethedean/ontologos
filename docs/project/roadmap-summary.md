# Roadmap Summary

OntoLogos follows [semantic versioning](https://semver.org/). **Current release:** [v0.6.0](https://github.com/eddiethedean/ontologos/releases/tag/v0.6.0) · **On `main`:** v0.7 dependency-first adapters · **Next tag:** v0.7.0

## Shipped through v0.6

| Area | Status |
|------|--------|
| Core data model, JSON v2, ABox | Available |
| OWL/RDF parser (horned-owl), profile detection | Available |
| RDFS / RL / EL facades (stable crate names) | Available |
| CLI `profile`, `materialize`, `classify`, `explain` | Available |
| Python alpha (`profile="rdfs"` / `"rl"` / `"el"` / `"auto"`) | Available |

## v0.7 on `main` (pending tag)

Dependency-first orchestration: **whelk** (EL), **reasonable** (RL/RDFS), **ontologos-bridge**, petgraph query/explain views. Custom in-house rule engines removed. CI gates: Pizza EL golden, Family RL triple closure.

## Next releases

| Version | Theme | Key deliverables |
|---------|-------|------------------|
| **0.7.0** | Adapter release | Tag + publish `ontologos-bridge`; facade crates |
| **0.8** | Incremental + petgraph polish | reasonable incremental wrapper; EL delta classify |
| **0.9** | Python ecosystem | PyPI wheels, full bindings |
| **1.0** | Stable release | Semver-stable facades; documented upstream gaps |
| **1.3** | Ontocode / LSP | IDE integration |
| **1.5+** | Hybrid routing | MORe-style whelk + reasonable modules |
| **2.0** | Full OWL DL | Extend whelk/horned-owl kernel |

## Full roadmap

Exit criteria, upstream gap policy, and detailed checklists:

**[ROADMAP.md on GitHub](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md)**

Architecture and adapter policy: [dependency-first ADR](../internal/design/dependency-first.md). Long-term vision (Ontocode, OntoHub): [PLAN.md on GitHub](https://github.com/eddiethedean/ontologos/blob/main/PLAN.md).
