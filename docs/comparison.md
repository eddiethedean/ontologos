# Comparison with Existing Tools

Honest positioning for evaluators. OntoLogos v0.1 is **not** a drop-in HermiT replacement.

## Maturity matrix

| Capability | OntoLogos v0.1 | HermiT / ELK | Protégé | owlready2 |
|------------|----------------|---------------|---------|-----------|
| Load OWL files | No (v0.2) | Yes | Yes | Yes |
| OWL EL classification | No (v0.5) | Yes (ELK) | Via plugin | Yes |
| OWL RL reasoning | No (v0.4) | Partial | Via plugin | Partial |
| RDFS materialization | No (v0.3) | Yes | Yes | Yes |
| Embeddable Rust API | **Yes** | JVM only | Desktop IDE | Python |
| In-memory graph + JSON | **Yes** | No | No | Yes |
| Explanations | No (v0.6) | Yes (HermiT) | Yes | Limited |
| Production-ready | **Pre-release** | Yes | Yes | Yes |

## When to use OntoLogos today

- Embedding an ontology **data model** in Rust
- Evaluating the architecture and roadmap
- Contributing to an open-source Rust reasoner

## When to use incumbents

- **Protégé + HermiT/ELK:** interactive OWL editing and classification today
- **owlready2:** Python-centric OWL workflows with reasoning via Pellet/HermiT
- **RDFox:** high-performance DLS reasoning and materialization (commercial)

## OntoLogos target (1.0)

Replace JVM-bound **batch** reasoning in Rust/Python pipelines with native EL/RL/RDFS engines, CLI, and IDE integration (Ontocode). Full OWL DL deferred to 2.0.

See [ROADMAP.md](../ROADMAP.md) for milestone dates.
