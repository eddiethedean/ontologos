# Roadmap Summary

OntoLogos follows [semantic versioning](https://semver.org/). **Latest tagged release:** **v0.8.0** · **v0.9.0** is ready on `main` (Python ecosystem; tag pending)

## Shipped in v0.9.0 (ready on `main`)

| Area | Status |
|------|--------|
| Python `Ontology` / `OntologyBuilder` | Complete |
| Python `explain()` + trace limit docs | Complete |
| Python incremental mutations | Complete |
| Optional pandas/polars taxonomy export | Complete |
| Pizza EL golden pytest (CLI parity) | Complete |
| macOS + Linux Python CI | Complete |

## Shipped in v0.8.0

| Area | Status |
|------|--------|
| Axiom dirty tracking + `remove_axiom` | Complete |
| EL incremental classify (`ElSession`, partitions) | Complete |
| RL/RDFS reasonable incremental session | Complete |
| `ontologos-watch` library (Ontocode hook) | Complete |
| Asserted/inferred axiom tracking (removal correctness) | Complete |

## Next releases

| Version | Theme | Key deliverables |
|---------|-------|------------------|
| **1.0** | Stable release | Semver-stable facades; documented upstream gaps |
| **1.3** | Ontocode / LSP | IDE integration (uses `ontologos-watch`) |
| **1.4** | Python maturity | Windows CI, mypy, owlready2 migration |
| **2.0** | Full OWL DL | Extend horned-owl / Konclude-style kernel |

See [ROADMAP.md on GitHub](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md) for full milestone detail.
