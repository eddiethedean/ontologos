# Roadmap Summary

OntoLogos follows [semantic versioning](https://semver.org/). **Workspace version:** **1.0.0** · **Latest tagged release:** **v0.9.0** · See [Release status](release-status.md) for crates.io, PyPI, and preview status.

## Shipped in v1.0.0 (workspace on `main`)

| Area | Status |
|------|--------|
| Python `Ontology` / `OntologyBuilder` | Complete |
| Python `explain()` + trace limit docs | Complete |
| Python incremental mutations | Complete |
| Optional pandas/polars taxonomy export | Complete |
| Pizza EL golden pytest (CLI parity) | Complete |
| macOS + Linux Python CI | Complete |

## Path to 1.0 — Full HermiT parity

**1.0** is the JVM-free replacement for **HermiT** (OWL 2 DL), not merely a “stable EL/RL/RDFS” cut. Expressivity tracks **v1.5–v1.9** block the 1.0 gate:

| Track | Theme |
|-------|-------|
| **v1.5** | Hybrid profile / MORe-style module routing |
| **v1.6** | ABox & individual reasoning |
| **v1.7** | ALC expressivity |
| **v1.8** | OWL QL & structured queries |
| **v1.9** | `ontologos-dl` engine (preview → stable in 1.0) |

**1.0 exit criteria:** HermiT conformance Tiers A–C, stable `ontologos-dl`, `classify --profile dl`, OWLReasoner-equivalent API.

## After 1.0

| Version | Theme |
|---------|-------|
| **1.1–1.2** | Performance, CLI polish |
| **1.3–1.4** | Ontocode LSP, Python maturity |
| **2.0** | Beyond HermiT (Konclude-class performance, breaking API where needed) |

See [ROADMAP.md on GitHub](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md) for full milestone detail.
