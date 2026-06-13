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
| **Delegate hint** | `Reasoner::classify()` error directing you to profile-specific crates |
| **JSON snapshot v2** | Serialized ontology format (`to_json` / `from_json`) |

## Engines (v0.9.0)

| Term | Crate | Scope |
|------|-------|-------|
| **RDFS materialization** | `ontologos-rdfs` | TBox: transitive subClass/subProperty, domain/range inheritance |
| **OWL RL saturation** | `ontologos-rl` | RDFS pass + RL TBox/ABox rules |
| **OWL EL classification** | `ontologos-el` | Completion-based taxonomy |
| **Incremental session** | core + profile crates | Delta re-classify via `ReasonerConfig::incremental` |
| **Proof graph** | `ontologos-explain` | EL-first explanations; RL/RDFS partial |

## Related

- [Choosing an API](choosing-an-api.md)
- [Architecture](../architecture.md)
- [First ontology](../getting-started/first-ontology.md)
