# HermiT Architecture Study

## Overview

HermiT is a tableau-based OWL 2 DL reasoner written in Java. It was the reference DL reasoner for Protégé and many OWL tools. As of 2023–2026 it is **largely unmaintained** (last meaningful release ~2020; [Abicht 2023](https://arxiv.org/abs/2309.06888) classifies it as abandoned) but still bundled in Protégé 5.6.x.

**OntoLogos DL strategy:** use HermiT as a **secondary conformance cross-check** where it still runs; primary DL architecture and benchmarks should follow **Konclude** (hybrid saturation + tableau). See [konclude.md](konclude.md) and [landscape-2023.md](landscape-2023.md).

## Architecture

| Component | Role |
|-----------|------|
| **Normalizer** | Converts OWL axioms to internal normal form |
| **Tableau engine** | Expands class expressions via completion rules |
| **Dependency index** | Tracks which axioms affect which entities |
| **Taxonomy builder** | Extracts subsumption hierarchy from saturated state |
| **Explanation** | Computes justifications for inferences and unsat |

## Key algorithms

- **Hypertableau** (optional): Optimized tableau for large ontologies
- **Incremental classification**: Re-saturates only affected partitions after axiom changes
- **Unsatisfiability**: Detects clashes in expanded tableau

## Performance characteristics

- Polynomial for EL fragments; exponential worst-case for full DL
- Heavy JVM overhead and object allocation on load
- Strong on expressive ontologies; overkill for EL/RL-only corpora

## Implications for OntoLogos

1. **Do not replicate HermiT in 1.x** — full DL tableau is explicitly a 2.0 goal per ROADMAP.
2. **Borrow taxonomy output shape**: class hierarchy as `(sub, super)` pairs, equivalence classes, unsatisfiable set — this becomes `ontologos-el::Taxonomy` and query API in v0.5.
3. **Borrow explanation model**: proof trees with rule name + premises map directly to `ProofNode` in `ontologos-explain` (v0.6).
4. **Incremental reasoning (v0.7)** should track axiom dependencies like HermiT's index, keyed by `EntityId` and `AxiomId` from the core model.
5. **EL classification (v0.5)** should follow ELK-style completion rather than tableau for the EL profile.
