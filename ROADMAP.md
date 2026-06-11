# Ontologos Roadmap

Ontologos is a Rust-native ontology reasoner built to replace JVM-bound reasoning workflows with an embeddable engine, CLI, Python bindings, and future IDE integration.

This document tracks phased delivery toward **1.0**. For architecture and API details, see [SPEC.md](SPEC.md). For background and ecosystem vision, see [PLAN.md](PLAN.md).

## Status

| Phase | Status |
|-------|--------|
| Phase 0 – Research | Not started |
| Phase 1 – Core Platform | **In progress** (workspace skeleton landed) |
| Phase 2 – RDFS Engine | Not started |
| Phase 3 – OWL RL Engine | Not started |
| Phase 4 – OWL EL Classifier | Not started |
| Phase 5 – Explanation Engine | Not started |
| Phase 6 – Incremental Reasoning | Not started |
| Phase 7 – Language Server | Not started |
| Phase 8 – Python Ecosystem | Not started |
| Phase 9 – 1.0 Release | Not started |

**Current milestone:** repository skeleton with all workspace crates, CLI wiring, CI, and stub APIs.

---

## Ecosystem Vision

Ontologos is the reasoning layer in a broader Rust ontology stack:

| Project | Role |
|---------|------|
| **Ontologos** | Reasoning engine |
| **OntoIndex** | Ontology query and index engine |
| **Ontocode** | VS Code extension |
| **OntoHub** | Registry and collaboration platform |

---

## Goals

### Primary

1. Replace JVM-bound reasoning workflows
2. Provide embeddable Rust APIs
3. Support Python data science workflows
4. Enable IDE-native ontology development
5. Support large ontology repositories

### Non-Goals (1.x)

- Full OWL 2 DL parity with HermiT
- Distributed reasoning
- Triple store replacement

---

## Phase 0 – Research

Establish the technical foundation before engine implementation.

### Deliverables

- [ ] OWL 2 standards review
- [ ] HermiT architecture study
- [ ] ELK architecture study
- [ ] RDFox evaluation
- [ ] Benchmark corpus assembled under `benchmarks/`

### Benchmark Ontologies

- Pizza
- Family
- GALEN
- Gene Ontology
- SNOMED subsets

---

## Phase 1 – Core Platform

Build the shared data model, parsers, and profile detection that all engines depend on.

### `ontologos-core`

- [x] Workspace crate and stub API (`Ontology`, `Reasoner`, `EntityKind`, axiom types)
- [ ] IRI intern pool
- [ ] Full axiom model
- [ ] Ontology graph
- [ ] Entity registry
- [ ] Serialization layer

**Performance target:** ontology load under 500ms for medium ontologies with stable allocation patterns.

### `ontologos-parser`

- [x] Workspace crate and format detection stubs
- [ ] horned-owl integration
- [ ] OWL/XML
- [ ] RDF/XML
- [ ] Turtle
- [ ] Functional Syntax

### `ontologos-profile`

- [x] Workspace crate and report types
- [ ] OWL EL detection
- [ ] OWL RL detection
- [ ] OWL QL detection
- [ ] OWL DL detection
- [ ] Diagnostics for unsupported constructs

### Exit criteria

- Load real ontologies from disk into the core model
- Profile command returns accurate detection and diagnostics
- Unit tests for parser, core model, and profile detector

---

## Phase 2 – RDFS Engine

**Crate:** `ontologos-rdfs`

### Features

- [ ] `SubClassOf`
- [ ] `SubPropertyOf`
- [ ] Domain
- [ ] Range
- [ ] Type propagation
- [ ] Transitive closure

### Deliverables

- [ ] Reasoning report
- [ ] Materialized graph (`ontologos materialize`)
- [ ] Explanation traces (initial)

**Complexity goal:** O(n log n)

### Exit criteria

- Pass RDFS conformance tests
- Materialize command produces correct inferences on benchmark ontologies

---

## Phase 3 – OWL RL Engine

**Crate:** `ontologos-rl`

### Rules

- [ ] `equivalentClass`
- [ ] `equivalentProperty`
- [ ] `sameAs`
- [ ] `inverseOf`
- [ ] `transitiveProperty`
- [ ] `symmetricProperty`
- [ ] Disjointness

### Implementation

- [ ] Forward chaining
- [ ] Rule indexing (`HashMap<EntityId, Vec<TripleId>>`)
- [ ] Parallel rule execution

**Performance goal:** million-triple datasets.

### Exit criteria

- OWL RL benchmark suite passes
- Parallel execution shows measurable speedup on large inputs

---

## Phase 4 – OWL EL Classifier

**Crate:** `ontologos-el`

### Features

- [ ] Completion rules and saturation
- [ ] Taxonomy generation
- [ ] Existential restrictions
- [ ] Intersections
- [ ] Unsatisfiable class detection
- [ ] Equivalent class detection

### Deliverables

- [ ] Class hierarchy output
- [ ] Incremental classification support
- [ ] Query API in `ontologos-query` (subsumption, direct subclasses)

### Exit criteria

- Classify command produces correct taxonomy on EL benchmark ontologies
- Unsatisfiable and equivalent classes reported accurately

---

## Phase 5 – Explanation Engine

**Crate:** `ontologos-explain`

### Features

- [ ] Proof graph (`ProofNode`, rule + premises)
- [ ] "Why inferred?" explanations
- [ ] "Why inconsistent?" explanations
- [ ] Minimal justification extraction
- [ ] Human-readable traces
- [ ] JSON and graph export

### Exit criteria

- Explain command returns valid proof trees for benchmark inferences
- Explanations integrate with RDFS, RL, and EL engines

---

## Phase 6 – Incremental Reasoning

### Capabilities

- [ ] File watch mode
- [ ] Delta reasoning (re-classify changed axioms only)
- [ ] IDE integration APIs for Ontocode

### Exit criteria

- Incremental re-classification is faster than full re-classification on typical edit workloads

---

## Phase 7 – Language Server Integration

Support **Ontocode** with live reasoning feedback.

### Features

- [ ] Live diagnostics
- [ ] Autocomplete
- [ ] Hover explanations
- [ ] Consistency warnings

### Exit criteria

- Ontocode prototype consumes Ontologos APIs for at least diagnostics and hover

---

## Phase 8 – Python Ecosystem

**Crate:** `ontologos-py` (published as `ontologos` on PyPI)

### Features

- [x] PyO3 bindings skeleton (`Reasoner` class)
- [ ] Maturin build and PyPI packaging
- [ ] pandas integration
- [ ] polars integration
- [ ] Notebook workflow examples

### Exit criteria

- `pip install ontologos` works on Linux and macOS
- Classification and explanation APIs exposed to Python

---

## Phase 9 – 1.0 Release

### Requirements

- [ ] Stable Rust API
- [ ] Stable CLI (`profile`, `classify`, `materialize`, `explain`)
- [ ] Published documentation
- [ ] Benchmark suite with published results
- [ ] CI/CD (crates.io + PyPI releases)
- [ ] OWL profile conformance suite

### Performance targets

| Ontology size | Classification target |
|---------------|----------------------|
| Small | < 100ms |
| Medium | < 1s |
| Large | < 10s |

### Quality targets

- 90%+ test coverage
- No JVM dependency
- Full benchmark suite green in CI

---

## Beyond 1.0 (2.x)

Deferred to a future major release:

- Hypertableau engine
- Nominals
- Cardinality restrictions
- Datatype reasoning
- Full OWL DL support

---

## Success Metrics

### Technical

- 90%+ test coverage
- Full benchmark suite passing
- Zero JVM dependency in the reasoning path

### Community

- crates.io adoption
- Python package adoption
- VS Code / Ontocode integration users
