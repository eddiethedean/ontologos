# Roadmap Summary

OntoLogos follows [semantic versioning](https://semver.org/). **Latest release:** **v1.1.4** on crates.io/PyPI/npm/Wasmer · See [Release status](release-status.md).

## v1.1.4 shipped (2026-07-12)

Test-suite verification and conformance guard honesty.

| Area | Status |
|------|--------|
| Conformance entailment guards | **Hardened** (weak IRI-shape shortcut removed) |
| Shared semantic fixtures | **Added** (`benchmarks/data/semantic-fixtures.json`) |
| Rust / Python API | **Unchanged** — bump pins to `"1.1.4"` |

## v1.1.0 shipped (2026-07-04)

Multi-language native bindings over shared FFI.

| Area | Status |
|------|--------|
| Shared FFI (`ontologos-ffi`) | **Shipped** |
| Java / .NET / C/C++ | **Source-build** with CI smoke |
| Node / WASM | **Published** on [npm](https://www.npmjs.com/package/ontologos) (`ontologos`, `@ontologos/wasm`; WASM also on [Wasmer](https://wasmer.io/eddiethedean/ontologos)) |
| Rust / Python API | **Unchanged** — bump pins to `"1.1.0"` |

See [v1.0.x → v1.1.0 migration](../migration/v1.0.x-to-v1.1.0.md) and [Bindings overview](../guides/bindings-overview.md).

## v1.0.0 (2026-07-03)

**`parity_pct = 100%`** — HermiT parity milestone on gated corpora.

| Area | Status |
|------|--------|
| OWL 2 DL (`ontologos-dl`) | **Stable** |
| DLSafe SWRL | **Stable** |
| HermiT conformance @ 30s | **1048** active tests, blocking CI |

## After 1.1

The **1.1** line is current. Historical pre-1.0 expressivity work formerly
labeled “v1.5–v1.9” is tracked as **E1–E5** in the maintainer roadmap; those
labels are not future releases.

| Version | Reference parity | Theme |
|---------|------------------|-------|
| **1.2** | ELK, Konclude, Openllet | Unified ABox realization plus CLI/performance polish |
| **1.3** | Openllet, Stardog, ELK | DL/RL/EL explanations and minimal justifications |
| **1.4** | RDFox, GraphDB, Jena | General incremental Datalog rule engine |
| **1.5** | ELK | Parallel EL classification, realization, and complex-expression queries |
| **1.6** | ELK, Openllet, JFact | OWL API adapter, Protégé smoke, Python/LSP maturity |
| **1.7** | Konclude, RacerPro | Session-oriented OWLlink reasoning service |
| **1.8** | RDFox, GraphDB, Konclude | SPARQL subset, advanced rules, query provenance |
| **1.9** | All reference engines | Compatibility soak and frozen 2.0 benchmarks |
| **2.0** | Konclude | Large-ontology DL performance, nominal schemas, bounded abduction |

See [full milestone roadmap](https://github.com/eddiethedean/ontologos/blob/main/docs/internal/roadmap.md) (maintainer doc).
