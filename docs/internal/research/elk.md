# ELK Architecture Study

## Overview

ELK is a polynomial-time OWL 2 EL classifier. It uses completion rules to saturate an ontology and extract a class taxonomy. It is the standard reference for EL reasoning performance.

## Architecture

| Component | Role |
|-----------|------|
| **Normal form conversion** | Flattens complex axioms to EL normal form |
| **Completion graph** | Stores `(C, r, D)` edges for existentials |
| **Rule engine** | Applies EL completion rules until saturation |
| **Taxonomy extraction** | Reads subsumption from saturated graph |
| **Incremental module** | Re-classifies after ontology edits |

## Completion rules (simplified)

1. **Subsumption propagation**: If `C ⊑ D` and `D ⊑ E`, infer `C ⊑ E`
2. **Existential introduction**: If `C ⊑ ∃r.D`, add edge `(C, r, D)`
3. **Existential composition**: If `(C, r, D)` and `D ⊑ ∃s.E`, infer `(C, r∘s, E)` when applicable
4. **Intersection handling**: Decompose `C ⊓ D` subsumptions

## Performance

- Polynomial time and space for EL
- Handles ontologies with millions of classes (e.g. SNOMED subsets, GO)
- Written in Java; still faster than general DL reasoners on EL ontologies

## Implications for OntoLogos

1. **`ontologos-el` (v0.5)** should implement ELK-style completion rules, not tableau.
2. **Core indexes** from v0.1 (`subclass_of`, `superclass_of`) are the seed data for saturation; EL engine adds derived edges to a separate inference store or materialized overlay.
3. **Incremental classification (v0.7)** follows ELK's partition-based re-saturation; requires `AxiomId`-level change tracking.
4. **Benchmark targets**: Pizza and GALEN are EL ontologies; use them as primary EL conformance corpora.
5. **Output API**: `ElClassifier::classify()` returns `Taxonomy { subsumptions, equivalences, unsatisfiable }` matching ELK's report structure.
