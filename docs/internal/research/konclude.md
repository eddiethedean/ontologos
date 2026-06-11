# Konclude Architecture Study

## Overview

[Konclude](https://www.uni-ulm.de/en/konclude/) is a parallel, high-performance OWL 2 DL reasoner for SROIQV(D) — effectively full OWL 2 DL plus nominal schemas. Implemented in **C++**, actively maintained on [GitHub](https://github.com/konclude/Konclude). Won the majority of DL tracks at ORE 2013–2015; still top-tier in 2023 evaluations alongside HermiT.

Unlike HermiT, Konclude is **not OWL API–native** (OWLlink / CLI primary).

## Architecture

| Component | Role |
|-----------|------|
| **Tableau calculus** | Sound, complete DL expansion with blocking |
| **Coupled saturation** | Completion-based procedure integrated with tableau data structures |
| **Preprocessor** | Axiom normalization before reasoning |
| **Dependency tracking** | Records how consequences were derived (unsat caching, explanations) |
| **Parallel execution** | Multi-core shared-memory classification |
| **Completion graph caching** | Indexing for large ontologies |

## Key insight: pay-as-you-go hybrid

From Steigmiller et al. (2014): saturation handles EL-like fragments efficiently **inside** the DL engine. When few axioms use disjunction or other saturation-hostile constructs, most work stays in the fast path. This differs from MORe's **external** module split but achieves similar economics.

## Performance (published)

- ESWC 2023: top classification/realization on large BioPortal ontologies; strongest consistency checking.
- ORE: dominates DL categories; ELK still wins pure EL tracks.

## Implications for OntoLogos

1. **2.0 architectural north star** — prefer Konclude-style **coupled saturation + tableau**, not a straight HermiT hypertableau port.
2. **Reference harness** — use Konclude CLI for DL benchmark baselines (alongside HermiT where still runnable).
3. **Dependency tracking** — Konclude's derivation tracking informs v0.6 explanations and v1.9 unsat caching.
4. **Do not target OWL API compatibility** — Konclude succeeds via OWLlink; OntoLogos should expose Rust/Python/CLI surfaces instead.
5. **Parallelism** — validates `ReasonerConfig::parallelism` for DL, not only RL.

## Non-goals

- Replicating Konclude's SROIQV nominal schemas in 2.0 initial scope.
- Matching Konclude's C++ micro-optimizations in first DL release.
