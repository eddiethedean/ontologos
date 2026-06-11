
# OntoLogos Plan Document

> **Note:** [ROADMAP.md](ROADMAP.md) is the **canonical** release plan (semver 0.1 → 2.0), including crate publish policy, CLI unlock schedule, exit criteria, and cross-cutting tracks.
> This document retains background and ecosystem vision. For current milestone status, use ROADMAP.

## Executive Summary

OntoLogos is a Rust-native ontology reasoner designed to eliminate JVM dependencies for ontology development workflows while providing a modern embeddable API, CLI, Python bindings, and future IDE integration through Ontocode.

**Research update (2026-06):** A survey of the OWL 2 reasoner landscape ([landscape-2023.md](docs/internal/research/landscape-2023.md)) confirms that most JVM DL reasoners (HermiT, Pellet, FaCT++) are **no longer actively maintained**, while **ELK** (EL) and **Konclude** (DL) remain the performance references. Rust peers **whelk-rs** (EL) and **reasonable** (RL) already exist — OntoLogos differentiates by shipping a **maintained, modular, multi-profile stack** (MORe-style hybrid routing in 1.5+) rather than being first-to-Rust.

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

- Full OWL 2 DL parity with HermiT or Konclude on day one of 2.0
- OWL API / JVM interoperability as a design center
- Distributed reasoning
- Triple store replacement

---

# Research findings (2023–2026)

Canonical survey: [docs/internal/research/landscape-2023.md](docs/internal/research/landscape-2023.md)

### Revelation 1 — Maintenance gap validates the mission

[Abicht 2023](https://arxiv.org/abs/2309.06888) reviewed 95 OWL reasoners; HermiT, Pellet, and CEL are effectively abandoned. Protégé 5.6.x still bundles HermiT, but users report hangs, OOM, and no fixes. **New projects should not depend on JVM DL reasoners for long-term maintenance.** OntoLogos targets the gap: embeddable, open, actively developed.

### Revelation 2 — Reference engines shift

| Profile | Old assumption | Updated reference |
|---------|----------------|-------------------|
| EL | ELK | **ELK** + **whelk-rs** (Rust peer) |
| RL | RDFox (commercial) | **reasonable** + OWLRL (open); RDFox aspirational |
| DL | HermiT hypertableau port | **Konclude** hybrid saturation+tableau; HermiT secondary cross-check |
| Hybrid | `Profile::Auto` picks one engine | **MORe**-style module/signature split (v1.5) |

### Revelation 3 — Hybrid composition is mandatory

Real biomedical ontologies are **mostly EL with occasional expressive axioms**. MORe (Oxford) proves that module extraction + black-box engine composition outperforms forcing a single reasoner. Konclude achieves similar pay-as-you-go economics by **coupling** saturation inside the tableau engine.

**Plan impact:** v1.5 must implement ⊥-module or signature partitioning over `ontologos-core`, not smarter single-profile detection alone.

### Revelation 4 — Rust ecosystem is not empty

- **horned-owl** — parse/manipulate ally (v0.2)
- **whelk-rs** — experimental EL classifier (INCATools / OBO community)
- **reasonable** — active OWL RL reasoner with Python bindings

OntoLogos competes on **unified profiles + CLI + Python + Ontocode + hybrid routing**, not on being the only Rust option.

### Revelation 5 — ELK implementation details matter

ELK uses goal-directed Closure/Todo saturation, parallel rules, ELK-specific taxonomy transitive reduction, partition-based incremental classification **without bookkeeping**, and built-in explanations. Naive completion or full re-classify will not match ELK on SNOMED/GO-scale corpora.

**Plan impact:** v0.5 EL engine and v0.7 incremental must follow Kazakov et al. literature (see [elk.md](docs/internal/research/elk.md)).

### Revelation 6 — IDE path over Protégé plugin

HermiT's stagnation as the default Protégé reasoner increases value for **Ontocode** (LSP, incremental classify) rather than investing in a Protégé plugin.

### Architectural decisions (from research)

1. **Parse with horned-owl; reason with ontologos-*** — do not re-export OWL API types.
2. **Conformance harnesses** — ELK/whelk-rs (EL), reasonable/OWLRL (RL), Konclude CLI (DL preview/2.0).
3. **2.0 engine** — Konclude-style coupled saturation + tableau, not HermiT-only port.
4. **Benchmark manifest** — add hybrid test ontologies for v1.5 (EL + sparse DL axioms).

### Research deliverables

| Document | Topic |
|----------|-------|
| [landscape-2023.md](docs/internal/research/landscape-2023.md) | Full ecosystem survey |
| [konclude.md](docs/internal/research/konclude.md) | DL architecture reference |
| [more.md](docs/internal/research/more.md) | Modular hybrid routing |
| [rust-ecosystem.md](docs/internal/research/rust-ecosystem.md) | whelk-rs, reasonable, horned-owl |
| [elk.md](docs/internal/research/elk.md) | EL algorithms (updated) |
| [hermit.md](docs/internal/research/hermit.md) | Legacy DL reference (updated) |

---

# Roadmap

See [ROADMAP.md](ROADMAP.md) for the authoritative semver release plan.

## Phase 0 – Research (complete)

### Deliverables

- OWL 2 standards review → [docs/internal/research/owl2.md](docs/internal/research/owl2.md)
- Reasoner landscape survey → [docs/internal/research/landscape-2023.md](docs/internal/research/landscape-2023.md)
- HermiT architecture study → [docs/internal/research/hermit.md](docs/internal/research/hermit.md)
- ELK architecture study → [docs/internal/research/elk.md](docs/internal/research/elk.md)
- Konclude architecture study → [docs/internal/research/konclude.md](docs/internal/research/konclude.md)
- MORe modular reasoner study → [docs/internal/research/more.md](docs/internal/research/more.md)
- Rust ecosystem study → [docs/internal/research/rust-ecosystem.md](docs/internal/research/rust-ecosystem.md)
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

For remaining phases (0.2–0.9, 1.1–1.9, 2.0), see [ROADMAP.md](ROADMAP.md).

## Phase 2 – Post-1.0 ladder (1.1 → 2.0)

After **1.0** stabilizes EL / RL / RDFS:

| Versions | Focus |
|----------|--------|
| **1.1–1.2** | Performance, CLI polish |
| **1.3–1.4** | Ontocode LSP, Python maturity |
| **1.5–1.7** | Hybrid profiles, ABox, ALC expressivity |
| **1.8** | OWL QL and structured queries |
| **1.9** | DL engine preview (`ontologos-dl`) |
| **2.0** | Full OWL 2 DL (stable) |
