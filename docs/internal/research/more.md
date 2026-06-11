# MORe Modular Reasoner Study

## Overview

[MORe](https://www.cs.ox.ac.uk/isg/tools/MORe/) (Modular OWL Reasoner) from Oxford ISG combines profile-specific engines with a full DL reasoner via **ontology module extraction**. Open-source (academic license), Java/OWL API.

Integrates **ELK** (EL), **RDFox** (RL/datalog), and **HermiT** (DL) as pluggable black boxes.

## Algorithm (simplified)

Given ontology O:

1. Identify **EL signature** Σ_EL where ELK is complete on the ⊥-module for Σ_EL.
2. Classify the EL module with ELK.
3. Classify the **residual signature** Sig(O) \ Σ_EL with HermiT (or Pellet).
4. Merge taxonomies.

## Properties

| Property | Detail |
|----------|--------|
| Pay-as-you-go | Adding one DL axiom does not force full DL on entire ontology |
| Black-box | Engines unchanged internally |
| TBox focus | ABox assertions ignored for classification completeness |
| Flexible | Any OWLReasoner factory can substitute for HermiT |

## Implications for OntoLogos

1. **v1.5 is research-validated** — hybrid `Profile::Auto` should implement MORe-style **module/signature splitting**, not pick a single profile for the whole ontology.
2. **Engine crate boundaries** — our `ontologos-el`, `ontologos-rl`, `ontologos-dl` split mirrors MORe's black-box composition; `Reasoner` facade orchestrates like MOReReasoner.
3. **Scope honesty** — initial hybrid mode may be TBox-only; document ABox limits like MORe until v1.6.
4. **Benchmark strategy** — hybrid corpora (EL + few DL axioms) should be added to `benchmarks/manifest.toml` for v1.5 exit criteria.
5. **No OWL API** — reimplement module extraction in Rust over `ontologos-core` (classic ⊥-module or structural splitting).

## References

- Armas Romero, A., Cuenca Grau, B., Horrocks, I. (2012). *MORe: Modular Combination of OWL Reasoners for Ontology Classification*. ISWC 2012.
- CEUR Vol-1015 system description.
