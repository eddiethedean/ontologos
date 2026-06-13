# Roadmap Summary

OntoLogos follows [semantic versioning](https://semver.org/). **Latest tagged release:** **v0.6.1** · **Next release:** **v0.7.0** (ready on `main`)

## Ready in v0.7.0 (tag + crates.io / PyPI publish)

| Area | Status |
|------|--------|
| Core data model, JSON v2, ABox | Available |
| OWL/RDF parser (horned-owl), profile detection | Available |
| RDFS / RL / EL facades (stable crate names) | Available |
| CLI `profile`, `materialize`, `classify`, `explain` | Available |
| Python alpha (`profile="rdfs"` / `"rl"` / `"el"` / `"auto"`) | Available |
| **`ontologos-explain`** + proof graphs | Available |
| **`ontologos-bridge`** (core ↔ horned-owl/oxrdf/reasonable) | Available |

## v0.7 — dependency-first adapters (v0.7.0)

| Area | Status |
|------|--------|
| **`ontologos-bridge`**; in-house EL; RL/RDFS → reasonable | Complete |
| Custom RL/RDFS rule engines removed | Complete |
| petgraph query/explain views | Complete |
| CI: Pizza EL golden, Family RL triple closure, HermiT Tier A | Complete |

## Next releases

| Version | Theme | Key deliverables |
|---------|-------|------------------|
| **0.8** | Incremental + petgraph polish | reasonable incremental wrapper; EL delta classify |
| **0.9** | Python ecosystem | PyPI wheels, full bindings |
| **1.0** | Stable release | Semver-stable facades; documented upstream gaps |
| **1.3** | Ontocode / LSP | IDE integration |
| **1.5+** | Hybrid routing | MORe-style EL + reasonable modules |
| **2.0** | Full OWL DL | Extend horned-owl / Konclude-style kernel |

See [ROADMAP.md on GitHub](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md) for full milestone detail.
