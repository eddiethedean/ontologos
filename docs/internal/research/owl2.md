# OWL 2 Standards Review

## Overview

OWL 2 (Web Ontology Language) is a W3C family of ontology languages with formal semantics. OntoLogos targets profile-restricted subsets rather than full OWL 2 DL in the 1.x line.

## Key constructs

| Construct | Semantics | OntoLogos engine |
|-----------|-----------|-------------------|
| `owl:Class`, `rdfs:subClassOf` | Class taxonomy | RDFS (v0.3), EL (v0.5) |
| `owl:ObjectProperty`, `rdfs:subPropertyOf` | Property hierarchy | RDFS (v0.3) |
| `rdfs:domain`, `rdfs:range` | Property typing | RDFS (v0.3) |
| `owl:equivalentClass` | Class equivalence | RL (v0.4), EL (v0.5) |
| `owl:disjointWith` | Disjointness | RL (v0.4) |
| `owl:inverseOf` | Inverse properties | RL (v0.4) |
| `owl:TransitiveProperty` | Transitive closure | RL (v0.4) |
| `owl:someValuesFrom` | Existential restriction | EL (v0.5) |
| `owl:intersectionOf` | Class intersection | EL (v0.5) |
| Nominals, cardinality, datatypes | Full DL | Deferred to 2.0 |

## OWL 2 profiles

| Profile | Characteristics | Reasoning style |
|---------|----------------|-----------------|
| **EL** | Existential restrictions, subsumption | Polynomial completion (ELK-style) |
| **RL** | Rule-friendly axioms over RDF triples | Forward chaining |
| **QL** | Query rewriting over DB | SPARQL-oriented (OntoIndex scope) |
| **DL** | Full expressivity | Tableau (HermiT-style, 2.0) |

## Implications for OntoLogos

1. **Core axiom model (v0.1)** should store the RL/RDFS axiom shapes listed in SPEC.md as structured enums with `EntityId` references, not OWL API-style nested class expressions yet.
2. **Profile detection (v0.2)** walks axioms and flags constructs outside EL/RL/RDFS; use `AxiomIndex::by_kind` for fast scans.
3. **Engine selection** maps `Profile::Auto` to the most specific detected profile, falling back to RDFS when only RDFS constructs are present.
4. **Defer DL constructs** (nominals, universal restrictions, cardinality) to v2.0; profile detector should emit diagnostics, not errors, for unsupported constructs.
5. **Hybrid ontologies are the norm** — real corpora (GALEN, OBO) are mostly EL with occasional expressive axioms. MORe-style module routing (v1.5) is required; single-profile Auto selection is insufficient. See [more.md](more.md).
6. **Do not depend on OWL API** — it anchors the JVM stack OntoLogos replaces; use horned-owl for parsing only.
7. **Maintenance landscape** — incumbent JVM DL reasoners are largely stagnant; ELK and Konclude remain the maintained references. See [landscape-2023.md](landscape-2023.md).
