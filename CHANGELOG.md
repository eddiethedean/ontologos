# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-06-12

### Added

- **`ontologos-rdfs`**: TBox RDFS materialization (`subClassOf` / `subPropertyOf` closure, domain/range inheritance)
- `MaterializationReport` with per-rule inference counts and optional traces
- `materialize_reasoner(&mut Reasoner)` and `classify_reasoner(&mut Reasoner)` for `Profile::Rdfs`
- `Reasoner::ontology_mut()` for in-place materialization
- `OntologyBuilder::property_domain` / `property_range` helpers
- RDFS unit tests (including long transitive chains) and Family/Pizza corpus conformance tests
- `ontologos materialize` CLI with text and JSON report output
- DL profile diagnostics explaining mapped constructs that rule out EL/RL
- Python `Reasoner(path, profile="rdfs")` for RDFS materialization via `classify()`
- `ontologos-conformance` workspace harness with HermiT Tier-A RDFS test ports ([tests/hermit/](tests/hermit/))
- HermiT replacement strategy doc ([docs/internal/research/hermit-replacement.md](docs/internal/research/hermit-replacement.md))

### Changed

- `RdfsEngine::materialize` now takes `&mut Ontology` and returns a structured report
- `Reasoner::classify` now takes `&mut self` (delegates to `ontologos-rdfs` from CLI/Python for `Profile::Rdfs`)
- `ontologos classify` runs RDFS materialization (OWL EL/RL classification remains v0.5)
- Workspace version bumped to 0.3.0

### Fixed

- Parser path sandbox: reject prefix-bypass paths (`uploads_base` vs `uploads_base_evil`)
- Parser mapping: allow axioms at entity limit; `DeclareDatatype` no longer blocks class with same IRI
- RDFS inference traces now record premise axiom ids
- Profile detection docs and Pizza corpus expectations aligned with DL classification (658 mapped axioms)

### Published

- [ontologos-core](https://crates.io/crates/ontologos-core) **0.3.0**
- [ontologos-parser](https://crates.io/crates/ontologos-parser) **0.3.0**
- [ontologos-profile](https://crates.io/crates/ontologos-profile) **0.3.0**
- [ontologos-rdfs](https://crates.io/crates/ontologos-rdfs) **0.3.0** (new crate)

## [0.2.0] - 2026-06-11

### Added

- **`ontologos-parser`**: OWL/XML, RDF/XML, OWL Functional Syntax, and Turtle loading via horned-owl; `ParseLimits`; `ParseMeta` on loaded ontologies
- **`ontologos-profile`**: EL / RL / QL / DL profile detection with diagnostics
- New axiom variants: `SubClassOfExistential`, `SymmetricObjectProperty`, `ReflexiveObjectProperty`, `FunctionalObjectProperty`
- Parser fixtures and manifest-driven integration tests (Pizza, Family)
- `benchmarks/scripts/download.sh` for benchmark corpus download

### Changed

- `ontologos profile` loads OWL files and reports detected profile
- `ontologos_parser::load_ontology` is the supported file-load API (`Ontology::from_file` remains a stub on `ontologos-core`)
- Pizza/Family benchmark corpora: Family ontology vendored in-repo; Pizza downloaded from owlcs/pizza-ontology

### Published

- [ontologos-parser](https://crates.io/crates/ontologos-parser) and [ontologos-profile](https://crates.io/crates/ontologos-profile) on crates.io

## [0.1.0] - 2026-06-11

First release. Publishes **`ontologos-core`** to [crates.io](https://crates.io/crates/ontologos-core) only; engine crates remain workspace-internal until implemented.

### Added

- Adoption documentation: CONTRIBUTING, FAQ, docs index, JSON v2 schema, security guide, error reference, comparison guide, runnable `pizza_builder` example
- Research notes under `docs/internal/research/` (OWL 2, HermiT, ELK, RDFox)
- Benchmark ontology manifest at `benchmarks/manifest.toml`
- `ontologos-core` in-memory ontology model:
  - `InternPool` / `IriId` for deduplicated absolute IRIs
  - `EntityRegistry` with kind validation
  - Structured `Axiom` enum with validation
  - `AxiomStore` and `AxiomIndex` for engine-ready lookups
  - `Ontology` facade with `OntologyBuilder`
  - Versioned JSON serialization (`to_json` / `from_json`)
- Integration tests with `pizza_minimal` fixture
- Criterion benchmark for 10k-axiom serialize/deserialize

### Changed

- JSON snapshot format bumped to **version 2** (IRI-keyed entities and axioms; v1 rejected for untrusted input)
- Hardened deserialization: resource limits, `deny_unknown_fields`, duplicate entity/axiom handling
- IRI validation: allowlist (`http`, `https`, `urn`), reject control characters and dangerous schemes
- `detect_profile` returns `Err(NotImplemented)` instead of empty success
- CLI: propagate emit errors, human-readable `--format text`, removed `--format yaml`
- File loading routed through `ontologos-parser::load_ontology` with path validation
- Hardened test suite: semantic JSON round-trip, `add_axiom` index wiring, axiom/IRI/entity edge cases, parser format detection, CLI smoke test
- `Ontology::from_file` now returns `Error::ParseNotAvailable` (parsing lands in v0.2)
- Breaking: `AxiomKind` replaced by structured `Axiom` with entity references

[Unreleased]: https://github.com/eddiethedean/ontologos/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/eddiethedean/ontologos/releases/tag/v0.3.0
[0.2.0]: https://github.com/eddiethedean/ontologos/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/eddiethedean/ontologos/releases/tag/v0.1.0
