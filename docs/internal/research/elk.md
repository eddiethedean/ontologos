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

## Taxonomy extraction

After saturation, ELK **transitively reduces** subsumptions to direct parents per equivalence class (ORE 2012 Algorithm 3). Naive nested iteration over all superclasses is insufficient when equivalence classes are present.

## Explanations

Modern ELK exposes step-by-step explanations for consequences. OntoLogos `ProofGraph` (v0.6) should align with ELK explanation shape for EL inferences.

## Rust peer

[whelk-rs](https://github.com/INCATools/whelk-rs) implements Kazakov-style EL rules in Rust (Horned-OWL ecosystem). Use **ELK + whelk-rs** as dual conformance references for v0.5. See [rust-ecosystem.md](rust-ecosystem.md).

## Implications for OntoLogos

1. **`ontologos-el` (v0.5)** should implement ELK-style **goal-directed** completion (Closure/Todo), not tableau.
2. **Taxonomy extraction** must use ELK's transitive-reduction algorithm over equivalence classes, not raw saturated subsumptions.
3. **Core indexes** from v0.1 are seed data; EL engine stores derived `(C, r, D)` edges in an inference overlay.
4. **Incremental classification (v0.7)** must use ELK's **partition-based overdelete-rederive** (Kazakov ISWC 2013) — no per-derivation bookkeeping.
5. **Benchmark targets**: Pizza, GALEN, GO-subset; compare output to ELK and whelk-rs.
6. **Output API**: `Taxonomy { subsumptions, equivalences, unsatisfiable }` matching ELK report structure.
7. **Parallelism**: ELK applies rules concurrently; `ReasonerConfig::parallelism` should extend to EL after RL proves the pattern.
