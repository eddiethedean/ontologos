# Roadmap Summary

OntoLogos follows [semantic versioning](https://semver.org/). **Latest tagged release:** **v0.7.0** · **Next release:** **v0.8** incremental reasoning

## Shipped in v0.7.0

| Area | Status |
|------|--------|
| Core data model, JSON v2, ABox | Available |
| OWL/RDF parser (horned-owl), profile detection | Available |
| RDFS / RL / EL facades (stable crate names) | Available |
| CLI `profile`, `materialize`, `classify`, `explain` | Available |
| Python alpha (`profile="rdfs"` / `"rl"` / `"el"` / `"auto"`) | Available on PyPI |
| **`ontologos-explain`** + proof graphs | Available |
| **`ontologos-bridge`** (core ↔ horned-owl/oxrdf/reasonable) | Available |

## v0.7 — dependency-first adapters (shipped)

| Area | Status |
|------|--------|
| **`ontologos-bridge`**; in-house EL; RL/RDFS → reasonable | Complete |
| Custom RL/RDFS rule engines removed | Complete |
| petgraph query/explain views | Complete |
| CI: Pizza EL golden, Family RL triple closure, HermiT Tier A | Complete |
| crates.io (9 crates) + PyPI wheels | Complete |

## Next releases

| Version | Theme | Key deliverables |
|---------|-------|------------------|
| **0.8** | Incremental + petgraph polish | reasonable incremental wrapper; EL delta classify; axiom dirty tracking |
| **0.9** | Python ecosystem maturity | `Ontology` builder, `explain()` bindings, pandas/polars export |
| **1.0** | Stable release | Semver-stable facades; documented upstream gaps |
| **1.3** | Ontocode / LSP | IDE integration |
| **1.5+** | Hybrid routing | MORe-style EL + reasonable modules |
| **2.0** | Full OWL DL | Extend horned-owl / Konclude-style kernel |

See [ROADMAP.md on GitHub](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md) for full milestone detail.
