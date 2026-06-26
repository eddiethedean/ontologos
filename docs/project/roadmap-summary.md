# Roadmap Summary

OntoLogos follows [semantic versioning](https://semver.org/). **Workspace version:** **1.0.0** · **Latest tagged release:** **v0.9.0** · See [Release status](release-status.md) for crates.io, PyPI, and preview status.

## HermiT parity progress (~58%)

The **v1.0.0 git tag** is blocked until [HermiT parity phases](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md#hermit-parity-phases-path-to-v100-tag) reach **Phase 9** (`parity_pct = 100%` — zero `planned` Java + WG catalog cases).

| Phase | Status | Milestone |
|-------|--------|-----------|
| 0–1 | Complete | Metrics + harness integrity |
| 2 | In progress | Assertion harvest (~87 cases) |
| 3–5 | Complete | Engine gaps, WG fixtures, manual ports |
| 6 | Complete | Tier B classification corpora |
| 7 | Complete | Tier C corpora + HermiT JAR proof |
| 8 | In progress | Expressivity v1.5–v1.9 |
| 9 | Planned | Tag v1.0.0 |

Verify: `bash benchmarks/scripts/check-hermit-parity-phases.sh` (informational in CI until Phase 9)

## Shipped on `main` (pre-1.0.0 tag)

| Area | Status |
|------|--------|
| Python `Ontology` / `OntologyBuilder` | Complete |
| Python `explain()` + trace limit docs | Complete |
| Python incremental mutations | Complete |
| Optional pandas/polars taxonomy export | Complete |
| Pizza EL golden pytest (CLI parity) | Complete |
| macOS + Linux Python CI | Complete |
| 593 active conformance tests; automated release gates pass | Complete |

## Path to 1.0 — Full HermiT parity

**1.0** is the JVM-free replacement for **HermiT** (OWL 2 DL). Expressivity tracks **v1.5–v1.9** are **Phase 8** of the parity plan:

| Track | Theme |
|-------|-------|
| **v1.5** | Hybrid profile / MORe-style module routing |
| **v1.6** | ABox & individual reasoning |
| **v1.7** | ALC expressivity |
| **v1.8** | OWL QL & structured queries |
| **v1.9** | `ontologos-dl` engine (preview → stable in 1.0) |

**1.0 exit criteria:** Phase 9 — 100% in-scope catalog parity, HermiT Tiers A–C blocking in CI, stable `ontologos-dl`, OWLReasoner-equivalent API.

## After 1.0

| Version | Theme |
|---------|-------|
| **1.1–1.2** | Performance, CLI polish |
| **1.3–1.4** | Ontocode LSP, Python maturity |
| **2.0** | Beyond HermiT (Konclude-class performance, breaking API where needed) |

See [ROADMAP.md on GitHub](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md) for full milestone detail.
