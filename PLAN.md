
# Ontologos Plan Document

## Executive Summary

Ontologos is a Rust-native ontology reasoner designed to eliminate JVM dependencies for ontology development workflows while providing a modern embeddable API, CLI, Python bindings, and future IDE integration through Ontocode.

## Vision

Build the foundation of a complete Rust ontology ecosystem:

- Ontologos: Reasoning engine
- OntoIndex: Ontology query/index engine
- Ontocode: VS Code extension
- OntoHub: Registry and collaboration platform

## Goals

### Primary Goals

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

# Roadmap

## Phase 0 – Research

### Deliverables

- OWL 2 standards review
- HermiT architecture study
- ELK architecture study
- RDFox evaluation
- Benchmark corpus

### Benchmark Ontologies

- Pizza
- Family
- GALEN
- Gene Ontology
- SNOMED subsets

---

## Phase 1 – Core Platform

### ontologos-core

Features:

- IRI intern pool
- Axiom model
- Ontology graph
- Entity registry
- Serialization layer

Performance Targets:

- <500ms ontology load for medium ontologies
- Stable memory allocation patterns

### ontologos-parser

Features:

- horned-owl integration
- OWL/XML
- RDF/XML
- Turtle
- Functional Syntax

### ontologos-profile

Detect:

- OWL EL
- OWL RL
- OWL QL
- OWL DL

Output diagnostics explaining unsupported constructs.

---

## Phase 2 – RDFS Engine

Implement:

- SubClassOf
- SubPropertyOf
- Domain
- Range
- Type propagation
- Transitive closure

Deliverables:

- Reasoning report
- Materialized graph
- Explanation traces

---

## Phase 3 – OWL RL Engine

Implement rule-based reasoning.

Rules:

- equivalentClass
- equivalentProperty
- sameAs
- inverseOf
- transitiveProperty
- symmetricProperty
- disjointness

Performance Goals:

- Million-triple datasets
- Parallel rule execution

---

## Phase 4 – OWL EL Classifier

Features:

- Taxonomy generation
- Existential restrictions
- Intersections
- Unsatisfiable class detection
- Equivalent class detection

Deliverables:

- Hierarchy explorer
- Incremental classification

---

## Phase 5 – Explanation Engine

Support:

- Why inferred?
- Why inconsistent?
- Proof trees
- Minimal justification extraction

---

## Phase 6 – Incremental Reasoning

Capabilities:

- File watch mode
- Delta reasoning
- IDE integration APIs

---

## Phase 7 – Language Server Integration

Support Ontocode.

Features:

- Live diagnostics
- Autocomplete
- Hover explanations
- Consistency warnings

---

## Phase 8 – Python Ecosystem

Package:

- ontologos-python

Features:

- pandas integration
- polars integration
- notebook workflows

---

## Phase 9 – 1.0 Release

Requirements:

- Stable API
- Stable CLI
- Documentation
- Benchmarks
- CI/CD
- Conformance suite

---

# Success Metrics

Technical:

- 90%+ test coverage
- Full benchmark suite
- No JVM dependency

Community:

- Crates.io adoption
- Python adoption
- VS Code integration users

