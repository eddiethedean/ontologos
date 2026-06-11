
# Ontologos Technical Specification

## Overview

Ontologos is a modular Rust ontology reasoner supporting OWL EL, OWL RL, RDFS reasoning, explanation generation, and incremental classification.

---

# Architecture

## Workspace Layout

```text
ontologos/
├── crates/
│   ├── ontologos-core
│   ├── ontologos-parser
│   ├── ontologos-profile
│   ├── ontologos-rdfs
│   ├── ontologos-rl
│   ├── ontologos-el
│   ├── ontologos-query
│   ├── ontologos-explain
│   ├── ontologos-cli
│   └── ontologos-py
``

---

# Core Data Model

## Entity Types

```rust
pub enum EntityKind {
    Class,
    Individual,
    ObjectProperty,
    DataProperty,
    AnnotationProperty,
}
```

## Axiom Types

Supported:

- SubClassOf
- EquivalentClasses
- DisjointClasses
- ObjectPropertyDomain
- ObjectPropertyRange
- SubObjectPropertyOf
- InverseObjectProperties
- TransitiveObjectProperty

---

# Reasoner API

```rust
let ontology = Ontology::from_file("pizza.owl")?;

let reasoner = Reasoner::builder()
    .profile(Profile::Auto)
    .build(ontology)?;

reasoner.classify()?;
```

Builder Options:

```rust
pub struct ReasonerConfig {
    pub incremental: bool,
    pub explanations: bool,
    pub parallelism: usize,
}
```

---

# RDFS Engine

Algorithms:

- Graph closure
- Property propagation
- Type propagation

Complexity Goal:

- O(n log n)

---

# OWL RL Engine

Implementation:

- Forward chaining
- Rule indexing
- Parallel execution

Storage:

```rust
HashMap<EntityId, Vec<TripleId>>
```

---

# OWL EL Engine

Algorithms:

- Completion rules
- Saturation
- Taxonomy extraction

Outputs:

- Class hierarchy
- Equivalent classes
- Unsatisfiable classes

---

# Explanation Engine

Proof Graph

```rust
pub struct ProofNode {
    pub rule: String,
    pub premises: Vec<NodeId>,
}
```

Features:

- Human readable traces
- JSON export
- Graph export

---

# CLI Specification

Commands

```bash
ontologos profile ontology.owl
ontologos classify ontology.owl
ontologos materialize ontology.owl
ontologos explain ontology.owl
```

Outputs:

- text
- json
- yaml

---

# Python Bindings

Examples

```python
from ontologos import Reasoner

r = Reasoner("ontology.owl")

r.classify()
```

---

# Performance Targets

Small Ontologies:

- <100ms classification

Medium:

- <1 second

Large:

- <10 seconds

---

# Testing Strategy

Unit Tests

- parser
- profile detector
- rule engine

Integration Tests

- benchmark ontologies

Conformance

- OWL profile suites

Coverage Target

- 90%+

---

# Future 2.x

- Hypertableau engine
- Nominals
- Cardinality restrictions
- Datatype reasoning
- Full OWL DL support
