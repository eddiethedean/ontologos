# Roadmap Summary

OntoLogos follows [semantic versioning](https://semver.org/). **Latest tagged release:** **v1.0.0** · See [Release status](release-status.md) for crates.io, PyPI, and parity metrics.

## v1.0.0 shipped (2026-07-03)

**`parity_pct = 100%`** — HermiT parity milestone on gated corpora. Published to crates.io (12 crates) and PyPI.

| Phase | Status | Milestone |
|-------|--------|-----------|
| 0–9 | Complete | Catalog, WG, expressivity, publish + tag **v1.0.0** |

Verify: `bash benchmarks/scripts/check-hermit-parity-phases.sh` · `bash benchmarks/scripts/hermit-burndown.sh status`

## Shipped in v1.0.0

| Area | Status |
|------|--------|
| OWL 2 DL (`ontologos-dl`) | **Stable** on crates.io/PyPI |
| DLSafe SWRL | **Stable** |
| Python `profile="dl"` / `profile="swrl"` | **Stable** |
| HermiT conformance @ 30s | **1048** active tests, blocking CI |
| JSON snapshot v3 | Writers on 1.0.0; v2 read supported |

## After 1.0

| Version | Theme |
|---------|-------|
| **1.1–1.2** | Performance, CLI polish |
| **1.3–1.4** | Ontocode LSP, Python maturity |
| **2.0** | Beyond HermiT (Konclude-class performance, breaking API where needed) |

See [full milestone roadmap](https://github.com/eddiethedean/ontologos/blob/main/docs/internal/roadmap.md) (maintainer doc).
