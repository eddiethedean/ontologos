
# OntoLogos Technical Specification

> **Document status:** Mixed. Sections marked **(v0.1)** / **(v0.2)** / **(v0.3)** / **(v0.4)** reflect shipped crates.
> OWL EL taxonomy classification, explanations, and full Python bindings are **planned** — see [ROADMAP.md](ROADMAP.md).
> Last reviewed: 2026-06-12

## Overview

OntoLogos is a modular Rust ontology reasoner supporting OWL EL, OWL RL, RDFS reasoning, explanation generation, and incremental classification. **v0.4 ships ABox in core, OWL RL saturation (`ontologos-rl`), plus parsing, profile detection, and RDFS from v0.2–v0.3.**

---

# Architecture

## Workspace Layout

```text
ontologos/
├── crates/
│   ├── ontologos-core      (v0.1+)
│   ├── ontologos-parser    (v0.2)
│   ├── ontologos-profile   (v0.2)
│   ├── ontologos-rdfs      (v0.3)
│   ├── ontologos-rl        (v0.4)
│   ├── ontologos-el        (stub → v0.5)
│   ├── ontologos-query     (stub → v0.5)
│   ├── ontologos-explain   (stub → v0.6)
│   ├── ontologos-cli       (partial — profile, materialize, classify/RDFS)
│   └── ontologos-py        (alpha — load, profile=rdfs/rl → v0.9 full API)
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

## Axiom Types (v0.1–v0.4)

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
- AsymmetricObjectProperty *(v0.4)*
- EquivalentObjectProperties *(v0.4)*
- ClassAssertion *(v0.4)*
- ObjectPropertyAssertion *(v0.4)*
- SameIndividual *(v0.4)*
- DifferentIndividuals *(v0.4)*

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

# Reasoner API (v0.1 facade, v0.3+ delegate hints)

**Status:** `Reasoner::classify(&mut self)` returns `Error::NotImplemented` for EL/Auto until v0.5 and `Error::Message` (delegate hint) for `Profile::Rdfs` / `Profile::Rl`. Use `ontologos_rdfs` or `ontologos_rl` profile crates for materialization.

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

# RDFS Engine (v0.3)

TBox RDFS materialization via fixed-point forward chaining:

- `subClassOf` transitive closure
- `subPropertyOf` transitive closure
- Domain/range inheritance along the property hierarchy (RDFS 6/8)

ABox `rdf:type` propagation is handled by `ontologos-rl` (v0.4).

```rust
use ontologos_rdfs::RdfsEngine;

let mut ontology = ontologos_parser::load_ontology("family.owl")?;
let report = RdfsEngine::new().materialize(&mut ontology)?;
```

For `Profile::Rdfs`, use `ontologos_rdfs::classify_reasoner`. For `Profile::Rl`, use `ontologos_rl::classify_reasoner`. Core `Reasoner::classify` returns delegate hints for RDFS/RL and `NotImplemented` for EL/Auto.

Implementation: batch fixed-point forward chaining (all rules per round until saturation). Worklist optimization is deferred to v1.1 performance work.

---

# OWL RL Engine (v0.4)

`RlEngine::saturate` runs RDFS materialization then OWL RL TBox/ABox rule batches until fixed point. Entry points: `ontologos_rl::classify_reasoner`, `materialize_reasoner`.

```rust
use ontologos_rl::RlEngine;

let report = RlEngine::new(1).saturate(&mut ontology)?;
```

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

**Status:** `profile`, `materialize`, and `classify` (RDFS) work in v0.4; `classify` and `materialize` emit the same inference report (`status` differs). OWL RL saturation is via `ontologos-rl` (library) or Python `profile="rl"`. CLI profile routing for RL ships in v0.5. `explain` loads then fails at engine stub.

Commands:

```bash
ontologos profile ontology.owl
ontologos materialize ontology.owl # RDFS (v0.3+)
ontologos classify ontology.owl    # RDFS materialization; OWL EL taxonomy in v0.5
ontologos explain ontology.owl     # NotImplemented (v0.6)
```

Outputs:

- `text` — human-readable
- `json` — structured

(YAML removed; was never implemented.)

---

# Python Bindings (alpha v0.4, full API v0.9)

**Status:** Alpha on PyPI; v0.4 exposes load, RDFS materialization, and OWL RL saturation.

```python
from ontologos import Reasoner

# RDFS materialization
r = Reasoner("ontology.owl", profile="rdfs")
r.classify()

# OWL RL saturation
r = Reasoner("family.owl", profile="rl")
r.classify()

# Default profile raises not-implemented until OWL EL taxonomy (v0.5)
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
