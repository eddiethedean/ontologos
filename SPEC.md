
# OntoLogos Technical Specification

> **Document status:** Mixed. Sections marked **(v0.1)** are implemented in `ontologos-core`.
> Unmarked engine, CLI, and Python sections are **planned** — see [ROADMAP.md](ROADMAP.md).
> Last reviewed: 2026-06-11

## Overview

OntoLogos is a modular Rust ontology reasoner supporting OWL EL, OWL RL, RDFS reasoning, explanation generation, and incremental classification. **v0.1 implements the core data model and JSON v2 serialization only.**

---

# Architecture

## Workspace Layout

```text
ontologos/
├── crates/
│   ├── ontologos-core      (v0.1)
│   ├── ontologos-parser    (stub → v0.2)
│   ├── ontologos-profile   (stub → v0.2)
│   ├── ontologos-rdfs      (stub → v0.3)
│   ├── ontologos-rl        (stub → v0.4)
│   ├── ontologos-el        (stub → v0.5)
│   ├── ontologos-query     (stub → v0.5)
│   ├── ontologos-explain   (stub → v0.6)
│   ├── ontologos-cli       (stub → v0.2+)
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

## Axiom Types (v0.1)

Supported in storage and validation:

- SubClassOf
- EquivalentClasses
- DisjointClasses
- ObjectPropertyDomain
- ObjectPropertyRange
- SubObjectPropertyOf
- InverseObjectProperties
- TransitiveObjectProperty

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
// Planned (v0.2+ load, v0.5 classify):
let ontology = Ontology::from_file("pizza.owl")?; // v0.2

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

# CLI Specification (planned)

**Status:** Binary builds; all subcommands fail at ontology load until v0.2.

Commands:

```bash
ontologos profile ontology.owl
ontologos classify ontology.owl
ontologos materialize ontology.owl
ontologos explain ontology.owl
```

Outputs (v0.1 CLI):

- `text` — human-readable
- `json` — structured

(YAML removed; was never implemented.)

---

# Python Bindings (planned v0.9)

**Status:** Stub only; `Reasoner("ontology.owl")` fails at load until v0.2.

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

**v0.1:**

- Unit and integration tests in `ontologos-core`
- Security regression tests
- Criterion benchmark for 10k-axiom JSON round-trip

**Planned:**

- Parser, profile, engine unit tests
- Benchmark ontology integration (see `benchmarks/manifest.toml`)
- OWL profile conformance suites
- 90%+ coverage target at 1.0

---

# Future 2.x

- Hypertableau engine
- Nominals
- Cardinality restrictions
- Datatype reasoning
- Full OWL DL support
