
# OntoLogos Technical Specification

> **Document status:** Mixed. Sections marked **(v0.1)** / **(v0.2)** reflect shipped crates.
> Engine, full CLI, and Python sections beyond profile detection are **planned** — see [ROADMAP.md](ROADMAP.md).
> Last reviewed: 2026-06-11

## Overview

OntoLogos is a modular Rust ontology reasoner supporting OWL EL, OWL RL, RDFS reasoning, explanation generation, and incremental classification. **v0.2 ships the core data model, JSON v2 serialization, OWL file loading, and profile detection.**

---

# Architecture

## Workspace Layout

```text
ontologos/
├── crates/
│   ├── ontologos-core      (v0.1+)
│   ├── ontologos-parser    (v0.2)
│   ├── ontologos-profile   (v0.2)
│   ├── ontologos-rdfs      (stub → v0.3)
│   ├── ontologos-rl        (stub → v0.4)
│   ├── ontologos-el        (stub → v0.5)
│   ├── ontologos-query     (stub → v0.5)
│   ├── ontologos-explain   (stub → v0.6)
│   ├── ontologos-cli       (partial — profile in v0.2)
│   └── ontologos-py        (stub → v0.9)
```

---

# Core Data Model (v0.1)

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

## Axiom Types (v0.1–v0.2)

Supported in storage and validation:

- SubClassOf
- SubClassOfExistential *(v0.2)*
- EquivalentClasses
- DisjointClasses
- ObjectPropertyDomain
- ObjectPropertyRange
- SubObjectPropertyOf
- InverseObjectProperties
- TransitiveObjectProperty
- SymmetricObjectProperty *(v0.2)*
- ReflexiveObjectProperty *(v0.2)*
- FunctionalObjectProperty *(v0.2)*

## File loading (v0.2)

- `ontologos_parser::load_ontology` — OWL/XML, RDF/XML, Turtle, OWL Functional Syntax
- `ParseMeta` on loaded ontologies: `constructs`, `profile_constructs`, `warnings`, axiom counts
- `Ontology::from_file` on core remains `ParseNotAvailable` (parser dependency isolation)

## Profile detection (v0.2)

- `ontologos_profile::detect_profile` — EL / RL / QL / DL classification
- Hybrid contract: classify on `profile_constructs`; diagnostics include source-only constructs outside detected profile
- See [docs/guides/profile-detection.md](docs/guides/profile-detection.md)

## JSON Serialization (v0.1)

- Format version **2** only for `from_json`
- IRI-keyed entities and axioms
- See [docs/json-snapshot-v2.md](docs/json-snapshot-v2.md)

## Builder API (v0.1)

```rust
use ontologos_core::{Error, Ontology};

fn main() -> Result<(), Error> {
    let ontology = Ontology::builder()
        .class("http://example.org/Pizza")?
        .class("http://example.org/Food")?
        .subclass_of("http://example.org/Pizza", "http://example.org/Food")?
        .build()?;
    Ok(())
}
```

---

# Reasoner API (planned)

**Status:** `Reasoner::classify()` returns `Error::NotImplemented` in v0.1.

```rust
// v0.2 load, v0.5 classify:
let ontology = ontologos_parser::load_ontology(path::Path::new("pizza.owl"))?;

let reasoner = Reasoner::builder()
    .profile(Profile::Auto)
    .build(ontology)?;

reasoner.classify()?; // v0.5
```

Builder options (v0.1 struct exists; validation enforced):

```rust
pub struct ReasonerConfig {
    pub incremental: bool,
    pub explanations: bool,
    pub parallelism: usize, // must be 1..=64
}
```

---

# RDFS Engine (planned v0.3)

Algorithms:

- Graph closure
- Property propagation
- Type propagation

Complexity goal: O(n log n)

---

# OWL RL Engine (planned v0.4)

Implementation:

- Forward chaining
- Rule indexing
- Parallel execution

---

# OWL EL Engine (planned v0.5)

Algorithms:

- Completion rules
- Saturation
- Taxonomy extraction

Outputs:

- Class hierarchy
- Equivalent classes
- Unsatisfiable classes

---

# Explanation Engine (planned v0.6)

Proof graph:

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

**Status:** `profile` works in v0.2; other subcommands load then fail at engine stubs.

Commands:

```bash
ontologos profile ontology.owl
ontologos classify ontology.owl    # NotImplemented
ontologos materialize ontology.owl # NotImplemented
ontologos explain ontology.owl     # NotImplemented
```

Outputs:

- `text` — human-readable
- `json` — structured

(YAML removed; was never implemented.)

---

# Python Bindings (planned v0.9)

**Status:** Alpha placeholder on PyPI; Rust v0.2 APIs not yet exposed in Python.

```python
from ontologos import Reasoner

r = Reasoner("ontology.owl")
r.classify()
```

---

# Performance Targets (1.0)

| Ontology size | Classification target |
|---------------|----------------------|
| Small | < 100ms |
| Medium | < 1s |
| Large | < 10s |

v0.1 Criterion bench: `cargo bench -p ontologos-core` (serialize/deserialize).

---

# Testing Strategy

**v0.1–v0.2:**

- Unit and integration tests in `ontologos-core`, `ontologos-parser`, `ontologos-profile`, `ontologos-cli`
- Security regression tests
- Manifest-driven corpus tests (Pizza, Family)
- Mapping fixtures per OWL format
- Criterion benchmark for 10k-axiom JSON round-trip

**Planned:**

- Engine unit tests (RDFS, RL, EL)
- OWL profile conformance suites
- 90%+ coverage target at 1.0

---

# Future 2.x

- Hypertableau engine
- Nominals
- Cardinality restrictions
- Datatype reasoning
- Full OWL DL support
