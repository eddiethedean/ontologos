# Glossary

Terms used across OntoLogos documentation. For OWL background, see the [W3C OWL 2 primer](https://www.w3.org/TR/owl2-overview/).

## Ontology structure

| Term | Meaning |
|------|---------|
| **TBox** | Terminological axioms — classes, properties, subsumption (the taxonomy/schema) |
| **ABox** | Assertional axioms — individuals, class/property assertions, same/different |
| **IRI** | Internationalized Resource Identifier — unique name for an entity (`http://...`) |
| **Entity** | A registered class, individual, or property in the core model |
| **Axiom** | A structured statement stored in `AxiomStore` (e.g. `SubClassOf`) |

## Reasoning

| Term | Meaning |
|------|---------|
| **Materialization** | Forward-chaining inference that **adds axioms** to the ontology until rules are satisfied |
| **Saturation** | Materialization until a **fixed point** — no new axioms can be inferred |
| **Classification** | In OWL tools (ELK/HermiT), computing the **class taxonomy** (subsumption hierarchy). OntoLogos CLI `classify` routes by `--profile` to EL taxonomy, RL saturation, or RDFS materialization |
| **Profile** | OWL 2 subset (EL, RL, QL, DL) with defined computational properties |
| **Subsumption** | `A subClassOf B` — every instance of A is an instance of B |

## OntoLogos-specific

| Term | Meaning |
|------|---------|
| **Mapper output** | Axioms stored in core after parsing — what `axiom_count()` returns |
| **ParseMeta** | Parser scan metadata: mapped/skipped counts, warnings, construct sets |
| **`profile_constructs`** | Constructs from **mapped** axioms — drives detected EL/RL/QL/DL |
| **`constructs`** | Full source scan — used for **diagnostics** only |
| **Facade** | `ontologos-facade` — unified `classify()` routing for CLI, Python, and multi-profile Rust |
| **Core Reasoner stub** | `Reasoner::classify()` on `ontologos-core` returns an error in 1.0.0 — use `ontologos_facade::classify` ([Rust integration contract](rust-integration-contract.md)) |
| **JSON snapshot v3** | Current serialization format (`to_json` emits v3; `from_json` reads v2 and v3) |
| **JSON snapshot v2** | Legacy format; still readable |
| **`parity_pct`** | In-scope HermiT catalog harness completion — see [Evaluator scope](evaluator-scope.md) |

## Engines

| Term | Crate | Scope |
|------|-------|-------|
| **RDFS materialization** | `ontologos-rl` (`rdfs` module) | TBox: transitive subClass/subProperty, domain/range inheritance |
| **OWL RL saturation** | `ontologos-rl` | RDFS pass + RL TBox/ABox rules |
| **OWL EL classification** | `ontologos-el` | Completion-based taxonomy |
| **OWL DL classification** | `ontologos-dl` | Hybrid EL + saturation + tableau (`main` / 1.0.0) |
| **SWRL** | `ontologos-swrl` | DLSafe rules + DL consistency (`main` / 1.0.0) |
| **Incremental session** | core + profile crates | Delta re-classify via `ReasonerConfig::incremental` |
| **Proof graph** | `ontologos-explain` | EL-first explanations; RL/RDFS partial |

## Related

- [Choosing an API](choosing-an-api.md)
- [Profile stability matrix](profile-stability.md)
- [Architecture](../architecture.md)
- [First ontology](../getting-started/first-ontology.md)
