# Roadmap Summary

OntoLogos follows [semantic versioning](https://semver.org/). **Latest tagged release:** **v0.7.0** · **Next release:** **v0.8.0** incremental reasoning (on `main`)

## Shipped in v0.8.0 (on `main`)

| Area | Status |
|------|--------|
| Axiom dirty tracking + `remove_axiom` | Complete |
| EL incremental classify (`ElSession`, partitions) | Complete |
| RL/RDFS reasonable incremental session | Complete |
| `ontologos-watch` library (Ontocode hook) | Complete |
| CLI `--incremental`, Python `incremental=True` | Complete |

## Next releases

| Version | Theme | Key deliverables |
|---------|-------|------------------|
| **0.9** | Python ecosystem maturity | `Ontology` builder, `explain()` bindings, pandas/polars export |
| **1.0** | Stable release | Semver-stable facades; documented upstream gaps |
| **1.3** | Ontocode / LSP | IDE integration (uses `ontologos-watch`) |
| **2.0** | Full OWL DL | Extend horned-owl / Konclude-style kernel |

See [ROADMAP.md on GitHub](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md) for full milestone detail.
