# Roadmap Summary

OntoLogos follows [semantic versioning](https://semver.org/). **Latest release:** **v1.1.1** on crates.io/PyPI · See [Release status](release-status.md).

## v1.1.1 shipped (2026-07-04)

Multi-language native bindings over shared FFI.

| Area | Status |
|------|--------|
| Shared FFI (`ontologos-ffi`) | **Shipped** |
| Java / .NET / C/C++ / Node / WASM | **Source-build** with CI smoke |
| Rust / Python API | **Unchanged** — bump pins to `"1.1.1"` |

See [v1.0.x → v1.1.1 migration](../migration/v1.0.x-to-v1.1.1.md) and [Bindings overview](../guides/bindings-overview.md).

## v1.0.0 (2026-07-03)

**`parity_pct = 100%`** — HermiT parity milestone on gated corpora.

| Area | Status |
|------|--------|
| OWL 2 DL (`ontologos-dl`) | **Stable** |
| DLSafe SWRL | **Stable** |
| HermiT conformance @ 30s | **1048** active tests, blocking CI |

## After 1.1

| Version | Theme |
|---------|-------|
| **1.2** | Performance, CLI polish, binding registry publish |
| **1.3–1.4** | Ontocode LSP, Python maturity |
| **2.0** | Beyond HermiT (Konclude-class performance, breaking API where needed) |

See [full milestone roadmap](https://github.com/eddiethedean/ontologos/blob/main/docs/internal/roadmap.md) (maintainer doc).
