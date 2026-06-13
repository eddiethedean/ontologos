# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.0] - 2026-06-13

> **Note:** On `main`, untagged. Latest git tag remains **v0.6.1**.

### Changed

- **Semver:** release **0.7.0** aligns crate versions with ROADMAP v0.7 (dependency-first adapters); no public API changes from 0.6.1
- Documentation and install examples updated to `0.7.0`

### Fixed

- **EL validation:** `validate_el_profile` uses `el_classification_forbidden_in` (complex TBox constructs the completion engine cannot handle)
- **EL completion:** multiple `ObjectPropertyDomain` axioms per property are all applied
- **Parser:** horned-owl panics on malformed RDF/XML (e.g. duplicate `rdf:ID`) converted to `Error::Parse`
- **CLI:** materialization reports include `clashes`; `parse_meta` errors emit stderr warnings
- **Conformance:** `sub_and_super_roles` uses direct subproperty assertion; manifest marks reasonable upstream gaps as `ignored`
- **Bridge:** optional `MergeLimits::max_axioms` cap during RL/RDFS merge
- **Security:** expanded `security_regressions` tests for JSON limits documented in `docs/security.md`

## [0.6.1] - 2026-06-13

### Changed

- **EL engine:** restore in-house ELK-style completion in `ontologos-el`; remove git `whelk` dependency
- **`ontologos-bridge`:** horned-owl/oxrdf/reasonable adapters only (no whelk)
- Pizza EL golden baseline regenerated from in-house EL (`84` direct subsumptions)
- All library crates publishable to crates.io again (full publish order restored)

### Fixed

- **`ontologos-query`:** removed `ontologos-el` dev-dependency so the crate publishes cleanly
- **`ontologos-explain`:** `ProofGraph::is_acyclic` uses the same validator as `build_proof_graph`

## [0.6.0] - 2026-06-12

> **Note:** The `v0.6.0` tag shipped a brief **whelk** EL delegation experiment and **partial** crates.io publish (core, parser, profile, query only). **v0.6.1** is the corrected public release: in-house EL restored, full library publish. See [0.6.1] below.

### Added

- **`ontologos-explain`**: `ProofGraph`, `build_proof_graph`, `explain_with_profile`, `explain_rdfs`/`explain_rl`/`explain_el`
- **`ontologos-bridge`**: core ↔ horned-owl/oxrdf/whelk/reasonable adapters (workspace-only until whelk is on crates.io)
- **`InferenceTrace`** / **`TraceStep`** in `ontologos-core` for engine-agnostic explanation traces
- Query APIs: `explain_subsumption`, `explain_unsatisfiable` with EL-first HST pruning
- Human-readable `render_text` formatter for proof graphs
- CLI `ontologos explain --profile auto|el|rl|rdfs` (JSON + text output)
- Conformance: `explain_benchmarks.rs`; Pizza EL golden vs in-house EL (`compare-pizza-el-golden.sh`); Family RL triple closure vs reasonable (`compare-reasonable.sh`)
- **petgraph** taxonomy views in `ontologos-query` and proof-graph acyclic checks in `ontologos-explain`
- RDFS `MaterializationReport::clashes` forwarded from reasonable diagnostics

### Changed

- **Dependency-first adapters:** `ontologos-el` → **whelk**; `ontologos-rdfs` / `ontologos-rl` → **reasonable**; parsing remains **horned-owl**
- Custom in-tree EL completion, RDFS/RL rule engines, and RL `TripleIndex` removed; public facade crate names unchanged
- RDFS/RL `MaterializationReport.traces` renamed to `.trace` (`InferenceTrace`)
- `ReasonerConfig::explanations` honored on classify routes (traces empty until upstream exposes rule diagnostics)
- Pizza EL golden baseline regenerated from whelk; HermiT RL/RDFS tests document reasonable upstream gaps via `#[ignore]`

### Fixed

- **Bridge merge:** `owl:sameAs`, `owl:disjointWith`, and `owl:differentFrom` no longer mis-map as property assertions
- **Bridge merge:** existential blank-node IDs include filler; restriction triples reconstruct `SubClassOfExistential`
- **Bridge export:** `DisjointClasses` and `DifferentIndividuals` mapped to horned-owl (no longer silently dropped)
- Reflexive `sameAs` triples from reasonable skipped on merge (avoids invalid `SameIndividual` axioms)
- RL report previously dropped RDFS inference traces during saturation (pre-adapter)

### Documentation

- [Dependency-first ADR](docs/internal/design/dependency-first.md)
- [Migration v0.5.x → v0.6.0](docs/migration/v0.5.x-to-v0.6.0.md)
- Updated architecture, comparison, ROADMAP, and Python guide for adapter stack

### Breaking

- Per-rule `MaterializationReport::inferred_by_rule` counts are empty when using the reasonable adapter (upstream does not expose rule-level diagnostics yet)
- EL classification returns `Taxonomy` without mutating the ontology; RL/RDFS saturation merges inferred axioms into core

## [0.5.0] - 2026-06-12

### Added

- **`ontologos-el`**: OWL EL completion classifier (`ElClassifier`, `classify_reasoner`, `classify_with_profile`)
- **`ontologos-query`**: taxonomy query API (`direct_subclasses`, `is_subsumed`, `equivalent_classes`, `unsatisfiable_classes`)
- **`Taxonomy`** type in `ontologos-core` (subsumptions, equivalences, unsatisfiable)
- CLI `--profile el|rl|rdfs|auto` on `classify`; taxonomy JSON/text output for EL
- Python `profile="el"` / `"auto"`; `classify()` returns taxonomy or materialization dict
- Parser: decompose `SubClassOf(C, ObjectIntersectionOf(...))` into EL axioms
- HermiT `ClassificationTest` pizza harness with vendored fixtures (`benchmarks/data/hermit/`)
- Pizza EL golden conformance (`pizza-el-golden.json`, `compare-pizza-el-golden.sh` in CI)
- Vendored `go-subset.owl` EL performance gate (< 10s)

### Changed

- **Breaking:** CLI `classify` defaults to `--profile auto` (EL/RL routing), not RDFS-only — use `materialize` or `--profile rdfs` for RDFS
- Python package version aligned to **0.5.0**

### Fixed

- **RL soundness:** replace unsound upward existential `subClassOf` propagation with downward scm-spo1
- **RL existential subsumption:** compare asserted existentials only (avoids spurious inferences from property weakening)
- **EL completion graph:** seed `EquivalentObjectProperties` at graph build time

### Documentation

- [OWL EL classification](docs/getting-started/owl-el-classification.md)
- [Migration v0.4.x → v0.5.0](docs/migration/v0.4.x-to-v0.5.0.md)
- v0.5 capability matrix, architecture, conformance, and Python guide updates

## [0.4.0] - 2026-06-12

### Added

- **ABox in `ontologos-core`**: `ClassAssertion`, `ObjectPropertyAssertion`, `SameIndividual`, `DifferentIndividuals`, `EquivalentObjectProperties`, `AsymmetricObjectProperty`
- Axiom indexes for individuals, property assertions, `sameAs`, and equivalent properties
- JSON snapshot v2 round-trip for new axiom variants; `OntologyBuilder` ABox helpers
- Parser mapping for named ABox axioms and `AsymmetricObjectProperty` / `EquivalentObjectProperties`
- **`ontologos-rl`**: OWL RL forward-chaining (`RlEngine::saturate`) on top of RDFS materialization
- `ontologos_rl::classify_reasoner` / `materialize_reasoner` for `Profile::Rl`
- HermiT Tier-A ports: `testSubsumption2` / `testSubsumption3` (inlined existential encoding)
- Family RL corpus test; optional `compare-reasonable.sh` harness for external diff

### Changed

- Profile detection: mapped ABox constructs are allowed under OWL RL (family corpus → RL)
- Python `profile="rl"` routes through `ontologos_rl::classify_reasoner`
- `Reasoner::classify()` with `Profile::Rl` returns delegate hint (use `ontologos_rl`)
- CLI `classify` prints stderr note that it runs RDFS only; `explain` hidden from `--help` until v0.6

### Documentation

- Read the Docs site with adoption-focused guides (crates.io quick start, RDFS tutorial, capability matrix, glossary, performance, production integration)
- FAQ and onboarding fixes; README slimmed for evaluators
- MkDocs strict build enforced in CI

### Published

- [ontologos-core](https://crates.io/crates/ontologos-core) **0.4.0**
- [ontologos-parser](https://crates.io/crates/ontologos-parser) **0.4.0**
- [ontologos-profile](https://crates.io/crates/ontologos-profile) **0.4.0**
- [ontologos-rdfs](https://crates.io/crates/ontologos-rdfs) **0.4.0**
- [ontologos-rl](https://crates.io/crates/ontologos-rl) **0.4.0**

## [0.3.1] - 2026-06-12

### Changed

- `ontologos classify` emits the same inference report as `materialize` (`status: classified`, axiom counts, per-rule breakdown)
- Clarified `Reasoner::classify` documentation: CLI/Python use `ontologos_rdfs` for `Profile::Rdfs`

### Fixed

- `Reasoner::classify()` with `Profile::Rdfs` returns a delegate hint instead of generic `NotImplemented`
- FAQ, load guide, and CLI reference updated for Pizza → DL profile detection
- Documented OWL import non-resolution, RDFS materialization scope, and batch fixed-point engine (vs worklist)

### Published

- [ontologos-core](https://crates.io/crates/ontologos-core) **0.3.1**
- [ontologos-parser](https://crates.io/crates/ontologos-parser) **0.3.1**
- [ontologos-profile](https://crates.io/crates/ontologos-profile) **0.3.1**
- [ontologos-rdfs](https://crates.io/crates/ontologos-rdfs) **0.3.1**

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

[Unreleased]: https://github.com/eddiethedean/ontologos/compare/v0.6.1...HEAD
[0.7.0]: https://github.com/eddiethedean/ontologos/compare/v0.6.1...HEAD
[0.6.1]: https://github.com/eddiethedean/ontologos/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/eddiethedean/ontologos/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/eddiethedean/ontologos/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/eddiethedean/ontologos/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/eddiethedean/ontologos/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/eddiethedean/ontologos/releases/tag/v0.3.0
[0.2.0]: https://github.com/eddiethedean/ontologos/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/eddiethedean/ontologos/releases/tag/v0.1.0
