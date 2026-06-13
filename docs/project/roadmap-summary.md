# Roadmap Summary

OntoLogos follows [semantic versioning](https://semver.org/). **Latest on crates.io:** partial **v0.6.0** (core, parser, profile, query) · **Next tag:** **v0.6.1** · **On `main`:** v0.6.1

## Shipped through v0.5 (crates.io)

| Area | Status |
|------|--------|
| Core data model, JSON v2, ABox | Available |
| OWL/RDF parser (horned-owl), profile detection | Available |
| RDFS / RL / EL facades (stable crate names) | Available |
| CLI `profile`, `materialize`, `classify` | Available |
| Python alpha (`profile="rdfs"` / `"rl"` / `"el"` / `"auto"`) | Available |

## v0.6.x on `main` (v0.6.1 pending tag)

| Area | Status |
|------|--------|
| **`ontologos-explain`** + CLI `explain` | Ready |
| **`ontologos-bridge`**; in-house EL; RL/RDFS → reasonable | Ready |
| petgraph query/explain views | Ready |
| CI: Pizza EL golden, Family RL triple closure | Ready |

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
