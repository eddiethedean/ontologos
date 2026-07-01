# Roadmap Summary

OntoLogos follows [semantic versioning](https://semver.org/). **Workspace version:** **1.0.0** · **Latest tagged release:** **v0.9.0** · See [Release status](release-status.md) for crates.io, PyPI, and parity metrics.

## HermiT parity progress (100% catalog; publish pending)

**`parity_pct = 100%`** on `main` (`java_planned = 0`, `wg_planned = 0`). Blocking CI runs **450** Java axiom + **428** OWL WG tests @ 30s; `check-1.0-release-gates.sh` is green. **v1.0.0 git tag** remains blocked on crates.io + PyPI publish only.

| Phase | Status | Milestone |
|-------|--------|-----------|
| 0–7 | Complete | Catalog, WG, Tier B/C proof |
| 8 | Complete | Expressivity v1.5–v1.9 (documented waivers) |
| 9 | **Ready** | Publish + tag **v1.0.0** |

Verify: `bash benchmarks/scripts/check-hermit-parity-phases.sh` · `bash benchmarks/scripts/hermit-burndown.sh status`

## Shipped on `main` (pre-1.0.0 tag)

| Area | Status |
|------|--------|
| Python `Ontology` / `OntologyBuilder` | Complete |
| Python `explain()` + trace limit docs | Complete |
| Python incremental mutations | Complete |
| Optional pandas/polars taxonomy export | Complete |
| Pizza EL golden pytest (CLI parity) | Complete |
| macOS + Linux Python CI | Complete |
| **1009** active conformance tests; release gates **blocking** | Complete |
| Full HermiT + WG suite @ 30s in PR CI | Complete |

## Path to 1.0 — Full HermiT parity

**1.0** is the JVM-free replacement for **HermiT** (OWL 2 DL) on the gated conformance catalog. Expressivity tracks **v1.5–v1.9** are **complete** (Phase 8):

| Track | Theme |
|-------|-------|
| **v1.5** | Hybrid profile / MORe-style module routing |
| **v1.6** | ABox & individual reasoning |
| **v1.7** | ALC expressivity |
| **v1.8** | OWL QL & structured queries |
| **v1.9** | `ontologos-dl` engine (stable on `main`; publish pending) |

**1.0 exit criteria remaining:** crates.io publish, PyPI **1.0.0**, annotated tag **v1.0.0**.

## After 1.0

| Version | Theme |
|---------|-------|
| **1.1–1.2** | Performance, CLI polish |
| **1.3–1.4** | Ontocode LSP, Python maturity |
| **2.0** | Beyond HermiT (Konclude-class performance, breaking API where needed) |

See [docs/internal/roadmap.md](internal/roadmap.md) for full milestone detail.
