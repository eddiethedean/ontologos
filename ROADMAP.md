# OntoLogos Roadmap

OntoLogos is a Rust-native ontology reasoner built to replace JVM-bound reasoning workflows with an embeddable engine, CLI, Python bindings, and future IDE integration.

Releases follow [semantic versioning](https://semver.org/). **0.x** builds capability toward **1.0**; **1.x** hardens and extends the stable API; **2.0** introduces full OWL DL reasoning.

For architecture and API details, see [SPEC.md](SPEC.md). For background and ecosystem vision, see [PLAN.md](PLAN.md).

**Last updated:** 2026-06-11 · **Current release:** [v0.2.0](https://github.com/eddiethedean/ontologos/releases/tag/v0.2.0) · **Next milestone:** v0.3 — RDFS engine

---

## How to read this document

| Symbol | Meaning |
|--------|---------|
| **Complete** | Shipped in a tagged release |
| **In progress** | Active or partially landed on `main` |
| **Planned** | Scoped but not started |
| **Deferred** | Explicitly out of scope for the named release |

Checklists use GitHub task syntax (`- [x]` / `- [ ]`) so progress is visible in diffs. Exit criteria are the release gate — a version ships when its criteria are met, not when every nice-to-have is done.

---

## Release overview

| Version | Theme | Crates unlocked | CLI commands | crates.io |
|---------|-------|-----------------|--------------|-----------|
| **0.1** | Core data model | `ontologos-core` | *(load fails)* | `ontologos-core` |
| **0.2** | Parsing & profiles | `+parser`, `+profile` | `profile` | `+parser`, `+profile` |
| **0.3** | RDFS engine | `+rdfs` | `materialize` | `+rdfs` |
| **0.4** | OWL RL engine | `+rl` | — | `+rl` |
| **0.5** | OWL EL & query | `+el`, `+query` | `classify` | `+el`, `+query` |
| **0.6** | Explanations | `+explain` | `explain` | `+explain` |
| **0.7** | Incremental reasoning | core, engines | — | — |
| **0.8** | LSP surface (Ontocode) | core API extensions | — | — |
| **0.9** | Python ecosystem | `+py` | — | PyPI `ontologos` |
| **1.0** | Stable release | all 0.x crates | all four | full set |
| **1.1** | Performance & benchmarks | engines | — | patch releases |
| **1.2** | CLI & export polish | cli | polish | — |
| **1.3** | Ontocode / LSP | `ontologos-lsp`? | — | optional crate |
| **1.4** | Python maturity | `ontologos-py` | — | PyPI |
| **1.5** | Profile & hybrid corpora | `profile`, engines | `--profile auto+` | — |
| **1.6** | ABox & individuals | core, `+abox`? | `instances` | TBD |
| **1.7** | ALC expressivity | `ontologos-alc` | — | TBD |
| **1.8** | OWL QL & queries | `ontologos-ql` | `query` | TBD |
| **1.9** | DL foundations | `ontologos-dl` (preview) | `classify --profile dl-preview` | TBD |
| **2.0** | Full OWL DL | `ontologos-dl` (stable) | `classify --profile dl` | `ontologos-dl` |

Workspace-internal crates (`ontologos-cli`) are not published; they consume the library crates above.

```mermaid
flowchart TB
  subgraph shipped [Shipped v0.1]
    core[ontologos-core]
  end

  subgraph v02 [v0.2]
    parser[ontologos-parser]
    profile[ontologos-profile]
  end

  subgraph engines [v0.3–v0.5]
    rdfs[ontologos-rdfs]
    rl[ontologos-rl]
    el[ontologos-el]
    query[ontologos-query]
  end

  subgraph surface [v0.6–v0.9]
    explain[ontologos-explain]
    cli[ontologos-cli]
    py[ontologos-py]
  end

  core --> parser
  parser --> profile
  core --> rdfs
  core --> rl
  core --> el
  el --> query
  rdfs --> explain
  rl --> explain
  el --> explain
  profile --> cli
  parser --> cli
  rdfs --> cli
  el --> cli
  explain --> cli
  core --> py
  parser --> py
```

---

## Design principles

1. **Core first** — All engines read and write through `ontologos-core`; no engine-specific ontology types.
2. **Fail honestly** — Unimplemented paths return typed errors (`NotImplemented`, `ParseNotAvailable`), not empty success.
3. **Benchmark-gated** — Each engine milestone must pass its corpus in [benchmarks/manifest.toml](benchmarks/manifest.toml) before release.
4. **Security by default** — Untrusted input (files, JSON) goes through validation and resource limits ([docs/security.md](docs/security.md)).
5. **Incremental publish** — Crates ship to crates.io when their API is stable enough for the milestone; the workspace may contain stubs earlier.

---

## Cross-cutting tracks

These run alongside version milestones and are not tied to a single release.

### Benchmarks & conformance

| Track | v0.1 | Target |
|-------|------|--------|
| Criterion serialize bench (10k axioms) | **Complete** | Keep in CI |
| OWL corpus manifest | **Complete** | Extend as engines land |
| Corpus download script | Planned (v0.2) | `benchmarks/scripts/download.sh` |
| Manifest-driven integration tests | Planned (v0.2) | Skip when `local_path` missing |
| Engine conformance suites | Planned (v0.3+) | Pizza, Family, GALEN, GO-subset |
| Criterion regression tracking in CI | Planned (v1.1) | Fail on >5% regression |

### Security & limits

| Track | v0.1 | Target |
|-------|------|--------|
| JSON v2 `Limits` for deserialization | **Complete** | Extend for file parse limits |
| IRI scheme allowlist | **Complete** | Maintain |
| Parser path traversal checks | **Complete** (stub path) | Keep for all load paths |
| Fuzzing / proptest for parser | Planned (v0.2) | OWL/XML + RDF/XML first |

### Documentation

| Track | v0.1 | Target |
|-------|------|--------|
| docs.rs for `ontologos-core` | **Complete** | Per published crate |
| JSON v2 schema doc | **Complete** | Keep in sync |
| Comparison guide | **Complete** | Update each milestone |
| Migration notes per release | Planned (v0.2+) | CHANGELOG + short upgrade guide |

---

## Ecosystem vision

OntoLogos is the reasoning layer in a broader Rust ontology stack:

| Project | Role | Relationship to OntoLogos |
|---------|------|---------------------------|
| **OntoLogos** | Reasoning engine | This repository |
| **OntoIndex** | Query and index engine | Consumes classified ontologies |
| **Ontocode** | VS Code extension | LSP client (v0.8 API surface) |
| **OntoHub** | Registry and collaboration | Distribution; out of scope for 1.0 |

---

## Goals

### Primary

1. Replace JVM-bound **batch** reasoning in Rust and Python pipelines
2. Provide embeddable, allocation-conscious Rust APIs
3. Support Python data science workflows (PyPI package)
4. Enable IDE-native ontology development via Ontocode
5. Handle medium-to-large ontologies (GO-scale subsets, not full SNOMED in CI)

### Non-goals (1.x)

- Full OWL 2 DL parity with HermiT
- Distributed or federated reasoning
- Triple store or SPARQL endpoint replacement
- Interactive ontology editing (delegated to Protégé / Ontocode)

### Comparison baseline

See [docs/comparison.md](docs/comparison.md) for an honest maturity matrix vs HermiT, ELK, Protégé, and owlready2.

---

# 0.x — Pre-release

## v0.1 — Core data model

**Status: Complete** ([v0.1.0](https://github.com/eddiethedean/ontologos/releases/tag/v0.1.0), 2026-06-11)

Establish the in-memory ontology representation all engines share.

### Research

- [x] OWL 2 standards review → [docs/internal/research/owl2.md](docs/internal/research/owl2.md)
- [x] HermiT architecture study → [docs/internal/research/hermit.md](docs/internal/research/hermit.md)
- [x] ELK architecture study → [docs/internal/research/elk.md](docs/internal/research/elk.md)
- [x] RDFox evaluation → [docs/internal/research/rdfox.md](docs/internal/research/rdfox.md)
- [x] Reasoner landscape survey → [docs/internal/research/landscape-2023.md](docs/internal/research/landscape-2023.md)
- [x] Konclude, MORe, Rust ecosystem studies → [konclude.md](docs/internal/research/konclude.md), [more.md](docs/internal/research/more.md), [rust-ecosystem.md](docs/internal/research/rust-ecosystem.md)
- [x] Benchmark corpus manifest → [benchmarks/manifest.toml](benchmarks/manifest.toml)

### `ontologos-core`

- [x] `InternPool` / `IriId` with validation and scheme allowlist
- [x] `EntityRegistry` with kind validation (`Class`, `Individual`, properties)
- [x] Structured `Axiom` enum with validation
- [x] `AxiomStore` (deduplicating) and `AxiomIndex` (subclass, subproperty, equivalence, inverse, …)
- [x] `Ontology` facade and `OntologyBuilder`
- [x] JSON snapshot **v2** (`to_json` / `from_json` / `from_json_with_limits`)
- [x] `Reasoner` / `ReasonerBuilder` API skeleton (`classify()` → `NotImplemented`)
- [x] Criterion benchmark: 10k-axiom serialize/deserialize
- [x] Integration tests, security regressions, `pizza_minimal` fixture

### Workspace stubs at v0.1 (superseded by v0.2 for parser/profile/cli)

- [x] `ontologos-rdfs`, `ontologos-rl`, `ontologos-el`, `ontologos-query`, `ontologos-explain` — typed stubs
- [x] `ontologos-py` — PyO3 `Reasoner` skeleton

### Exit criteria (met)

- [x] `ontologos-core` published to crates.io
- [x] JSON v2 round-trip tests green
- [x] `cargo test --workspace` and `cargo clippy -D warnings` pass in CI

---

## v0.2 — Parsing & profile detection

**Status: Complete** ([v0.2.0](https://github.com/eddiethedean/ontologos/releases/tag/v0.2.0), 2026-06-11) · **Depends on:** v0.1

Load real ontologies from disk, map them into the core model, and report which OWL profile they fall into.

### Phase A — Parser foundation

**Crate:** `ontologos-parser`

- [x] Format detection by extension and content sniffing
- [x] Path normalization and traversal rejection
- [x] `horned-owl` dependency and error mapping
- [x] OWL/XML reader
- [x] RDF/XML reader
- [x] Horned-owl → `ontologos-core` axiom mapping layer
- [x] `load_ontology` entry point (core `Ontology::from_file` remains stub by design)
- [x] Parse limits (max file size, max axioms) aligned with [docs/security.md](docs/security.md)

### Phase B — Additional formats

- [x] Turtle / `.ttl`
- [x] OWL Functional Syntax (`.ofn`, `.func`)
- [x] Unified `load_ontology` entry point used by CLI

### Phase C — Core extensions (as needed)

- [x] Audit horned-owl constructs against [SPEC.md](SPEC.md) axiom list
- [x] Add axiom variants: `SubClassOfExistential`, RL property declarations
- [x] Document unsupported constructs and emit parser warnings (`ParseMeta`)

### Phase D — Profile detection

**Crate:** `ontologos-profile`

- [x] `ProfileReport`, `ProfileDiagnostic`, `OwlProfile` types
- [x] Construct scanner over mapped axioms and `ParseMeta`
- [x] OWL EL / RL / QL / DL detection with hybrid diagnostics
- [ ] `ReasonerBuilder::profile(Profile::Auto)` reads detector (stub until v0.5 classify)

### Tooling & tests

- [x] `benchmarks/scripts/download.sh` for Pizza and Family corpora
- [x] Manifest-driven integration tests
- [x] Parser mapping tests per format
- [x] Profile unit tests and hybrid diagnostics tests

### CLI

- [x] `ontologos profile <file>` — text and JSON output
- [x] Remaining subcommands load ontology then fail at engine (`NotImplemented`)

### Exit criteria (met)

- [x] `load_ontology` loads Pizza and Family into core without panic
- [x] Parsed axiom counts within 10% of manifest `axiom_count_approx`
- [x] `ontologos profile` reports `El` for Pizza and `Rl` for Family
- [x] `ontologos-parser` and `ontologos-profile` published to crates.io
- [x] No new `unsafe` (workspace lint enforced)

### Risks

| Risk | Mitigation |
|------|------------|
| `horned-owl` construct coverage gaps | Map supported axioms first; diagnostics for the rest |
| Large files (GO) exhaust memory | Parse limits; CI uses `go-subset` only |
| Complex class expressions in EL corpora | Store for profile detection; full EL reasoning is v0.5 |

---

## v0.3 — RDFS engine

**Status: Planned** · **Effort:** Medium · **Depends on:** v0.2

**Crate:** `ontologos-rdfs`

First reasoning engine. Implements RDFS entailment over the core axiom model.

### Rules

- [ ] `rdfs:subClassOf` propagation (transitive closure)
- [ ] `rdfs:subPropertyOf` propagation
- [ ] `rdfs:domain` / `rdfs:range` typing
- [ ] `rdf:type` propagation where representable in core

### Implementation

- [ ] `RdfsEngine::materialize` produces inferred axioms or a materialized view
- [ ] `Reasoner::classify` with `Profile::Rdfs` delegates here
- [ ] Complexity target: O(n log n) on benchmark corpora

### Deliverables

- [ ] Materialization report (counts of new axioms by rule)
- [ ] `ontologos materialize <file>` — text status and JSON summary
- [ ] Initial inference traces (feeds v0.6 explain)

### Exit criteria

- [ ] RDFS conformance tests pass on Family corpus
- [ ] Materialized Pizza ontology is a strict superset of parsed axioms
- [ ] `ontologos-rdfs` published to crates.io

---

## v0.4 — OWL RL engine

**Status: Planned** · **Effort:** Large · **Depends on:** v0.3

**Crate:** `ontologos-rl`

Forward-chaining OWL RL rules on top of RDFS materialization.

### Rules (OWL 2 RL / RDF-Based Semantics)

- [ ] `equivalentClass` / `equivalentProperty`
- [ ] `sameAs` / `differentFrom` (where in RL fragment)
- [ ] `inverseOf`
- [ ] `TransitiveProperty`, `SymmetricProperty`, `AsymmetricProperty`
- [ ] `hasKey`, property chain axioms (RL subset)
- [ ] Disjointness propagation

### Implementation

- [ ] `RlEngine::saturate` fixed-point forward chaining
- [ ] `TripleIndex` rule indexing (`HashMap<EntityId, Vec<…>>`)
- [ ] Parallel rule batches via `ReasonerConfig::parallelism`
- [ ] `Reasoner::classify` with `Profile::Rl` delegates here

### Exit criteria

- [ ] RL conformance tests pass on Family corpus
- [ ] Materialized output matches **[reasonable](https://github.com/gtfierro/reasonable)** and OWLRL on Family + Brick subset fixtures
- [ ] Parallel mode shows measurable speedup on synthetic 100k-triple fixture
- [ ] `ontologos-rl` published to crates.io

> **Research:** [rust-ecosystem.md](docs/internal/research/rust-ecosystem.md) — `reasonable` is the active open Rust RL peer; RDFox remains aspirational for performance.

---

## v0.5 — OWL EL classifier & query

**Status: Planned** · **Effort:** Large · **Depends on:** v0.2 (parse); v0.4 optional for hybrid corpora

**Crates:** `ontologos-el`, `ontologos-query`

Completion-based EL classification — the primary use case for biomedical ontologies.

### `ontologos-el`

- [ ] Goal-directed saturation (Closure / Todo queues per ELK — see [elk.md](docs/internal/research/elk.md))
- [ ] Normal form conversion for EL axioms
- [ ] Completion rules until fixpoint
- [ ] Taxonomy extraction with transitive reduction over equivalence classes (ELK ORE 2012 Algorithm 3)
- [ ] Existential restrictions (∃R.C)
- [ ] Intersections (⊓)
- [ ] Unsatisfiable class detection
- [ ] Equivalent class clustering
- [ ] `ElClassifier::classify` returns `Taxonomy`
- [ ] `Reasoner::classify` with `Profile::El` / `Profile::Auto` delegates here

### `ontologos-query`

- [ ] `QueryEngine::direct_subclasses` over classified taxonomy
- [ ] Subsumption queries (A ⊑ B?)
- [ ] Equivalent class lookup
- [ ] Unsatisfiable class listing

### CLI

- [ ] `ontologos classify <file>` — taxonomy summary (text + JSON)
- [ ] `--profile el|rl|rdfs|auto` routes to correct engine

### Exit criteria

- [ ] Pizza EL taxonomy matches **ELK** and **[whelk-rs](https://github.com/INCATools/whelk-rs)** reference output (modulo ordering)
- [ ] GALEN or `go-subset` classifies within performance budget (< 10s for subset)
- [ ] `ontologos-el` and `ontologos-query` published to crates.io

> **Research:** ELK is the maintained EL gold standard; whelk-rs is the Rust conformance peer. Do not use HermiT as EL benchmark.

---

## v0.6 — Explanation engine

**Status: Planned** · **Effort:** Medium · **Depends on:** v0.3–v0.5

**Crate:** `ontologos-explain`

### Features

- [x] `ProofGraph`, `ProofNode`, `NodeId` types
- [ ] Record rule applications during RDFS / RL / EL runs (`ReasonerConfig::explanations`)
- [ ] "Why inferred?" for subclass and subsumption queries
- [ ] "Why inconsistent?" for unsatisfiable classes
- [ ] Minimal justification extraction (HST-style pruning — EL first)
- [ ] Human-readable trace formatter
- [ ] JSON export (`ProofGraph::to_json` exists; populate graphs)

### CLI

- [ ] `ontologos explain <file>` — JSON proof graph; text defers to `--format json`

### Exit criteria

- [ ] Explanations generated for ≥ 10 benchmark inferences across three engines
- [ ] Proof graphs are acyclic and reference valid axiom ids
- [ ] `ontologos-explain` published to crates.io

---

## v0.7 — Incremental reasoning

**Status: Planned** · **Effort:** Medium · **Depends on:** v0.5

Avoid full re-classification on small ontology edits.

### Capabilities

- [ ] Axiom-level dirty tracking in core
- [ ] EL: partition-based overdelete-rederive (Kazakov ISWC 2013 — no per-derivation bookkeeping)
- [ ] Incremental RL saturation for rule additions
- [ ] `ReasonerConfig::incremental` flag
- [ ] File-watch API for Ontocode (library only; CLI `--watch` may land in v1.2)

### Exit criteria

- [ ] Incremental EL re-classification is ≥ 5× faster than full classify on 10-axiom delta for Pizza
- [ ] Correctness: incremental taxonomy equals full classify taxonomy on SNOMED-scale random ±1/±10/±100 axiom edits (ELK methodology)

> **Research:** ELK incremental design in [elk.md](docs/internal/research/elk.md); naive full re-classify will not scale to GO/SNOMED edit workloads.

---

## v0.8 — Language server surface (Ontocode)

**Status: Planned** · **Effort:** Medium · **Depends on:** v0.5, v0.7

Ontocode lives in a separate repository; this milestone defines the **API contract** OntoLogos exposes.

### Library APIs

- [ ] `OntologyDelta` type for edit notifications
- [ ] Incremental classify hook with cancellation
- [ ] Diagnostic struct (severity, range, message, related axiom ids)
- [ ] Hover payload (entity label, superclasses, unsat status)
- [ ] Stable `ontologos-lsp` crate or documented module in core (decision at implementation time)

### Exit criteria

- [ ] Ontocode prototype consumes APIs for live diagnostics and hover on Pizza
- [ ] API documented in SPEC.md and semver-guaranteed from 1.0

---

## v0.9 — Python ecosystem

**Status: Planned** · **Effort:** Medium · **Depends on:** v0.2 (load), v0.5 (classify), v0.6 (explain)

**Crate:** `ontologos-py` · **PyPI name:** `ontologos`

### Features

- [x] PyO3 `Reasoner` skeleton
- [ ] Maturin build and manylinux / macOS wheels
- [ ] `Ontology` construction from Python (builder or dict)
- [ ] `classify()`, `materialize()`, `explain()` bindings
- [ ] Optional pandas / polars export for taxonomies
- [ ] Notebook examples under `examples/python/`

### Exit criteria

- [ ] `pip install ontologos` works on Linux and macOS (aarch64 + x86_64)
- [ ] Python integration test classifies Pizza and matches Rust CLI output
- [ ] PyPI release in CI (manual approval gate)

---

# 1.0 — Stable release

**Status: Planned** · **Gate for production use**

All 0.x capabilities integrated, tested, documented, and semver-stable.

### Requirements

- [ ] `#![deny(missing_docs)]` on all published crates
- [ ] Stable Rust API with deprecation policy documented
- [ ] CLI: `profile`, `classify`, `materialize`, `explain` fully functional
- [ ] docs.rs complete for every published crate
- [ ] Benchmark suite with published results in [benchmarks/README.md](benchmarks/README.md)
- [ ] OWL profile conformance suite green in CI
- [ ] Automated crates.io + PyPI release workflow
- [ ] MSRV policy documented (currently 1.78+)

### Performance targets

| Corpus class | Axioms (approx.) | Classify target |
|--------------|------------------|-----------------|
| Small (Family) | < 100 | < 100 ms |
| Medium (Pizza) | ~ 800 | < 1 s |
| Large (go-subset) | ~ 10k | < 10 s |

### Quality targets

- ≥ 90% line coverage on published crates (measured in CI)
- Zero JVM dependency in the reasoning path
- Full workspace `clippy -D warnings` clean

---

# 1.x — Post-1.0 ladder (1.0 → 2.0)

Incremental releases after 1.0. **API breaking changes require 2.0.** Versions 1.1–1.4 harden the 1.0 platform; 1.5–1.9 extend expressivity toward full OWL DL so 2.0 is an integration release, not a greenfield rewrite.

```mermaid
flowchart LR
  v10[1.0 Stable EL/RL/RDFS]
  v11[1.1 Perf]
  v12[1.2 CLI]
  v13[1.3 LSP]
  v14[1.4 Python]
  v15[1.5 Hybrid profiles]
  v16[1.6 ABox]
  v17[1.7 ALC]
  v18[1.8 QL]
  v19[1.9 DL preview]
  v20[2.0 Full DL]

  v10 --> v11 --> v12
  v10 --> v13
  v10 --> v14
  v12 --> v15
  v15 --> v16 --> v17 --> v19
  v17 --> v18
  v19 --> v20
  v18 --> v20
```

| Phase | Versions | Theme |
|-------|----------|-------|
| **Hardening** | 1.1–1.2 | Performance, CLI, ops |
| **Ecosystem** | 1.3–1.4 | IDE and Python adoption |
| **Expressivity** | 1.5–1.7 | Richer OWL fragments toward DL |
| **Query** | 1.8 | OWL QL and structured queries |
| **DL prep** | 1.9 | Tableau scaffolding and preview |
| **DL** | 2.0 | Full OWL 2 DL |

---

## v1.1 — Performance & benchmarks

**Status: Planned** · **Effort:** Medium · **Depends on:** 1.0

- [ ] Criterion benchmarks in CI with regression tracking (fail on > 5% regression)
- [ ] Published results table for all standard corpora in [benchmarks/README.md](benchmarks/README.md)
- [ ] Memory profiling and hot-path allocation reduction in EL and RL engines
- [ ] `cargo bench` documented per published crate
- [ ] Load-time budget: Pizza parse + classify < 500 ms on reference hardware

### Exit criteria

- [ ] Benchmark CI job runs on every PR to `main`
- [ ] Published numbers for Pizza, Family, and `go-subset`

---

## v1.2 — CLI & export polish

**Status: Planned** · **Effort:** Small · **Depends on:** 1.0

- [ ] YAML output format (`--format yaml`)
- [ ] Richer text reporting for `classify` and `explain`
- [ ] `ontologos --watch` for incremental file reload (uses v0.7 incremental APIs)
- [ ] Shell completions (`clap_complete`)
- [ ] `--timeout` and `--parallelism` flags on classify

### Exit criteria

- [ ] All four subcommands support `--format json|yaml|text`
- [ ] Completions shipped for bash, zsh, and fish

---

## v1.3 — Ontocode integration

**Status: Planned** · **Effort:** Medium · **Depends on:** 1.0, v0.8 LSP APIs

- [ ] Stable LSP protocol surface (versioned separately from core semver)
- [ ] `ontologos-lsp` crate or documented `ontologos_core::lsp` module
- [ ] Ontocode extension published to VS Code marketplace
- [ ] Diagnostic and hover conformance test suite
- [ ] Cancellation tokens for long classify runs in IDE

### Exit criteria

- [ ] Ontocode v1 uses only documented OntoLogos APIs (no private crate internals)
- [ ] Pizza ontology: unsat warning and hover superclass list verified in CI

---

## v1.4 — Python maturity

**Status: Planned** · **Effort:** Medium · **Depends on:** 1.0, v0.9 Python bindings

- [ ] Windows wheel support (x86_64)
- [ ] Type stubs (`py.typed`) and `mypy` clean in examples
- [ ] Polars and pandas DataFrame export for taxonomies
- [ ] Async-friendly classify API (optional `asyncio` feature)
- [ ] Documented migration from owlready2 for batch EL workflows

### Exit criteria

- [ ] `pip install ontologos` on Windows, Linux, macOS (aarch64 + x86_64)
- [ ] Python classify output matches Rust CLI on Pizza integration test

---

## v1.5 — Profile completeness & hybrid corpora

**Status: Planned** · **Effort:** Large · **Depends on:** 1.0

Real ontologies mix EL-safe TBox with RL/DL axioms. **MORe** (Oxford) proves module-based black-box composition outperforms single-reasoner selection — see [more.md](docs/internal/research/more.md).

### Module routing (`Reasoner` facade)

- [ ] ⊥-module or signature extraction over `ontologos-core` (Rust-native; no OWL API)
- [ ] Classify EL module with `ontologos-el`; RL residue with `ontologos-rl`; DL residue with `ontologos-dl` preview (when available)
- [ ] Merge taxonomies from module results
- [ ] TBox-first scope (ABox deferred to v1.6, matching MORe initial semantics)

### `ontologos-profile`

- [ ] Hybrid ontology report: EL / RL / DL construct partitions per module
- [ ] `Profile::Auto` invokes MORe-style orchestration, not single-engine pick

### Engines

- [ ] RL rule completeness audit against OWL 2 RL spec
- [ ] Document soundness/completeness trade-offs per module
- [ ] Add hybrid test ontologies to `benchmarks/manifest.toml`

### Exit criteria

- [ ] GALEN hybrid report: EL module classifies without false DL delegation on EL-safe fragment
- [ ] Hybrid corpus taxonomy matches MORe (ELK+HermiT) or Konclude reference within documented tolerance

---

## v1.6 — ABox & individual reasoning

**Status: Planned** · **Effort:** Large · **Depends on:** 1.5

**Crates:** `ontologos-core` extensions, optional `ontologos-abox`

Full DL requires individual assertions. EL/RL pipelines also benefit from typed instances and `sameAs` closure.

### Core extensions

- [ ] ABox axiom types: `ClassAssertion`, `ObjectPropertyAssertion`, `DataPropertyAssertion`
- [ ] `NegativePropertyAssertion` (RL subset)
- [ ] Individual typing propagation integrated with RL engine
- [ ] `sameAs` / `differentFrom` closure (RL and ABox modules)

### `ontologos-abox` (if not folded into RL)

- [ ] Instance typing report
- [ ] Consistency check for asserted individuals
- [ ] CLI: `ontologos instances <file>` — list types and conflicts

### Exit criteria

- [ ] Family corpus: all asserted individuals typed correctly after materialize
- [ ] `sameAs` chain closure matches RL reference on synthetic fixture
- [ ] ABox axioms round-trip through JSON v3 (schema bump; v2 remains supported for TBox-only)

---

## v1.7 — ALC expressivity (pre-DL TBox)

**Status: Planned** · **Effort:** Large · **Depends on:** 1.6

**Crate:** `ontologos-alc`

Bridge between EL completion and full tableau: **ALC** (attributive language with complement) — unions, negation, and universal restrictions without nominals or cardinality.

### Features

- [ ] Internal normal form for ALC class expressions
- [ ] Universal restrictions (∀R.C)
- [ ] Unions and complements in class expressions (stored or normalized on load)
- [ ] Tableau-lite saturation for ALC (single global tableau, no hypertableau yet)
- [ ] Unsatisfiability under ALC semantics
- [ ] `Reasoner::classify` with `Profile::Alc` (new variant, non-breaking if enum is `#[non_exhaustive]`)

### Exit criteria

- [ ] ALC benchmark suite (standard literature ontologies + synthetic) passes vs reference
- [ ] Pizza + ALC extension axioms: unsat detected where expected
- [ ] Documented boundary: ALC in 1.7, not full DL

---

## v1.8 — OWL QL & structured queries

**Status: Planned** · **Effort:** Large · **Depends on:** 1.5, 1.7

**Crate:** `ontologos-ql`

OWL QL supports query answering via rewriting over EL/RL class hierarchies. Integrates with **OntoIndex** for embeddable query workflows.

### Features

- [ ] OWL QL profile detection refinement (conjunctive query shapes)
- [ ] Conjunctive query AST and parser (functional or SPARQL subset — decision at implementation)
- [ ] Query rewriting over classified taxonomy
- [ ] `QueryEngine` extensions: instance retrieval, conjunctive query answering
- [ ] CLI: `ontologos query <file> --query '<cq>'` (JSON result rows)
- [ ] Stable C API or FFI surface for OntoIndex consumption (optional)

### Exit criteria

- [ ] QL conformance tests from W3C OWL 2 QL test cases (subset documented in SPEC)
- [ ] Query answering on Pizza + ABox extensions matches reference engine

---

## v1.9 — DL engine foundations (preview)

**Status: Planned** · **Effort:** Very large · **Depends on:** 1.7, 1.8

**Crate:** `ontologos-dl` (preview, semver 0.x within workspace until 2.0)

Scaffolding for full DL without committing to 2.0 API stability. Users opt in via feature flag or `--profile dl-preview`.

### Infrastructure (Konclude hybrid model — see [konclude.md](docs/internal/research/konclude.md); HermiT as secondary cross-check in [hermit.md](docs/internal/research/hermit.md))

- [ ] OWL axiom normalizer → internal DL normal form
- [ ] **Coupled saturation + tableau** (pay-as-you-go; not pure hypertableau port)
- [ ] Dependency index keyed by `EntityId` / `AxiomId` (derivation tracking for unsat cache + explain)
- [ ] Tableau expansion core (branching, clash detection, blocking)
- [ ] Taxonomy extraction from saturated tableau
- [ ] **Konclude CLI** + HermiT reference harness in `benchmarks/`

### Preview fragment (ALCH + nominals subset)

- [ ] Role hierarchy (H) integrated with ALC tableau from 1.7
- [ ] Nominals (individuals in class expressions) — limited count per ontology
- [ ] `classify --profile dl-preview` behind explicit CLI warning
- [ ] Explanations for DL preview inferences (reuse v0.6 graph)

### Exit criteria

- [ ] DL preview classifies ≥ 3 published DL benchmark ontologies within 10× **Konclude** time (HermiT secondary where runnable)
- [ ] No panics on DL benchmark corpus; timeouts return structured errors
- [ ] 2.0 RFC issue drafted with API stabilization plan

### Decision criteria (promote preview → 2.0)

- [ ] `ontologos-dl` preview stable for ≥ 3 months without breaking internal APIs
- [ ] Reference harness covers Pizza-DL, Galen-DL subset, and one OBO DL corpus
- [ ] Maintainer sign-off on multi-year support commitment for full DL

---

# 2.0 — Full OWL DL

**Status: Planned** · **Major release** · **Depends on:** 1.9

Promotes `ontologos-dl` from preview to stable. **2.0 is integration and completeness**, not a restart — coupled saturation+tableau lands in 1.9 per [konclude.md](docs/internal/research/konclude.md).

### Scope (complete OWL 2 DL)

- [ ] Hypertableau or Konclude-style tableau optimizations (optional `ReasonerConfig` flag)
- [ ] Full nominal support (unbounded)
- [ ] Cardinality and qualified cardinality restrictions
- [ ] Datatype reasoning (OWL 2 datatypes subset: XSD primitives used in OWL)
- [ ] Full disjointness, keys, and property chains in DL semantics
- [ ] `classify --profile dl` — stable, no preview warning
- [ ] DL explanations parity with EL quality bar

### Performance targets

| Corpus class | Target |
|--------------|--------|
| Medium DL (≤ 5k axioms) | < 30 s classify |
| Large DL (Galen-class) | Best effort; timeout configurable |

### Exit criteria

- [ ] W3C OWL 2 DL test case suite (documented subset) passes above agreed threshold
- [ ] Comparison guide updated: OntoLogos 2.0 vs Konclude (+ HermiT where applicable) on standard corpora
- [ ] `ontologos-dl` published to crates.io with stable API

### Non-goals (carried forward)

- Distributed reasoning
- Triple store or SPARQL endpoint replacement
- Bit-for-bit parity with every HermiT optimization
- OWL 2 Full (non-DL constructs beyond spec scope)

---

## Success metrics

### Technical (from 1.0 onward)

- ≥ 90% test coverage on published crates
- Full benchmark suite passing in CI on every PR
- Zero JVM dependency in the reasoning path
- No critical security advisories on parser or JSON deserialization

### Adoption

- `ontologos-core` downloads on crates.io
- PyPI install base for `ontologos`
- External contributors landing PRs against engine crates
- Ontocode / third-party LSP clients using the v0.8 API surface

### Community

- Issues and discussions reflect real ontology workflows (not just API bikeshedding)
- Comparison guide updated when milestones ship

---

## Changelog linkage

Release notes are recorded in [CHANGELOG.md](CHANGELOG.md). Each tagged version should update the roadmap status table at the top of this file.
