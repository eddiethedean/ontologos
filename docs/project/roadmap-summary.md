# Roadmap Summary

OntoLogos follows [semantic versioning](https://semver.org/). **Latest release:** **v0.6.1** (includes ROADMAP **v0.7** dependency-first adapters) · **On `main`:** v0.6.1

## Shipped through v0.6.1 (crates.io + PyPI)

| Area | Status |
|------|--------|
| Core data model, JSON v2, ABox | Available |
| OWL/RDF parser (horned-owl), profile detection | Available |
| RDFS / RL / EL facades (stable crate names) | Available |
| CLI `profile`, `materialize`, `classify`, `explain` | Available |
| Python alpha (`profile="rdfs"` / `"rl"` / `"el"` / `"auto"`) | Available |
| **`ontologos-explain`** + proof graphs | Available |
| **`ontologos-bridge`** (core ↔ horned-owl/oxrdf/reasonable) | Available |

## v0.7 — dependency-first adapters (shipped in v0.6.1)

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
