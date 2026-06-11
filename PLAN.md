
# OntoLogos Plan Document

> **Note:** [ROADMAP.md](ROADMAP.md) is the **canonical** release plan (semver 0.1 → 2.0).
> This document retains background and ecosystem vision. For current milestone status, use ROADMAP.

## Executive Summary

OntoLogos is a Rust-native ontology reasoner designed to eliminate JVM dependencies for ontology development workflows while providing a modern embeddable API, CLI, Python bindings, and future IDE integration through Ontocode.

## Vision

Build the foundation of a complete Rust ontology ecosystem:

- OntoLogos: Reasoning engine
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

See [ROADMAP.md](ROADMAP.md) for the authoritative semver release plan.

## Phase 0 – Research (complete)

### Deliverables

- OWL 2 standards review → [docs/internal/research/owl2.md](docs/internal/research/owl2.md)
- HermiT architecture study → [docs/internal/research/hermit.md](docs/internal/research/hermit.md)
- ELK architecture study → [docs/internal/research/elk.md](docs/internal/research/elk.md)
- RDFox evaluation → [docs/internal/research/rdfox.md](docs/internal/research/rdfox.md)
- Benchmark corpus → [benchmarks/manifest.toml](benchmarks/manifest.toml)

### Benchmark Ontologies

- Pizza
- Family
- GALEN
- Gene Ontology
- SNOMED subsets

---

## Phase 1 – Core Platform (v0.1 complete)

### ontologos-core

Features (shipped):

- IRI intern pool
- Axiom model
- Ontology graph
- Entity registry
- JSON v2 serialization layer

Performance targets:

- Stable memory allocation patterns
- Criterion bench for 10k-axiom serialize/deserialize

### ontologos-parser (v0.2)

Features:

- horned-owl integration
- OWL/XML
- RDF/XML
- Turtle
- Functional syntax

---

For remaining phases, see [ROADMAP.md](ROADMAP.md).
