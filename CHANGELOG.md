# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- PyPI package **`ontologos`** (0.1.0 pre-release placeholder): maturin build, `python/ontologos` package, publish script

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

[Unreleased]: https://github.com/eddiethedean/ontologos/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/eddiethedean/ontologos/releases/tag/v0.1.0
