# OWL 2 Reasoner Landscape (2023–2026)

Survey of the incumbent reasoner ecosystem and what it implies for OntoLogos.
Sources: [Abicht 2023](https://arxiv.org/abs/2309.06888), [ESWC 2023 DL reasoner evaluation](https://ceur-ws.org/Vol-3443/ESWC_2023_DMKG_paper_2861.pdf), [Manchester reasoner list](http://owl.cs.manchester.ac.uk/tools/list-of-reasoners/), project repositories (checked 2026-06).

## Executive summary

1. **Most JVM DL reasoners are effectively unmaintained** (HermiT, Pellet, FaCT++, CEL). They still ship inside Protégé but receive few fixes.
2. **ELK remains the EL gold standard** — actively maintained, polynomial, parallel, incremental, with explanations.
3. **Konclude is the modern DL benchmark** — C++, actively developed, hybrid tableau + saturation, top ORE results; not OWL API–native.
4. **Hybrid composition is proven** — MORe (ELK + RDFox + HermiT via modules) and Konclude (saturation + tableau) both use pay-as-you-go strategies OntoLogos should adopt in 1.5+.
5. **Rust peers already exist** — `reasonable` (RL), `whelk-rs` (EL), `horned-owl` (parse/manipulate). OntoLogos must differentiate on modular profiles + maintained full stack, not “first Rust reasoner.”

---

## Maintenance status (selected reasoners)

| Reasoner | Language | Profile | Maintenance (2023–2026) | Notes |
|----------|----------|---------|-------------------------|-------|
| **ELK** | Java | EL | **Active** ([liveontologies/elk-reasoner](https://github.com/liveontologies/elk-reasoner)) | Protégé default for EL; incremental + parallel |
| **Konclude** | C++ | DL (SROIQV) | **Active** ([konclude/Konclude](https://github.com/konclude/Konclude)) | Best DL benchmarks; OWLlink, not OWL API |
| **Openllet** | Java | DL | Partial ([Galigator/openllet](https://github.com/Galigator/openllet)) | Pellet fork; used in 2023 evals |
| **HermiT** | Java | DL | **Stagnant** ([owlcs/hermit-reasoner](https://github.com/owlcs/hermit-reasoner)) | Last release ~2020; bundled in Protégé 5.6.x |
| **Pellet** | Java | DL | **Stagnant** | Last release 2015 |
| **FaCT++** | C++ | DL | **Stagnant** | Last release 2016 |
| **RDFox** | C++ | RL/Datalog | Commercial | Reference for RL materialization performance |
| **MORe** | Java | Hybrid | Research / alpha | Modular ELK + RDFox + HermiT |
| **reasonable** | Rust | RL | **Active** ([gtfierro/reasonable](https://github.com/gtfierro/reasonable)) | DataFrog Datalog; Python + CLI |
| **whelk-rs** | Rust | EL | **Active** ([INCATools/whelk-rs](https://github.com/INCATools/whelk-rs)) | Horned-OWL ecosystem; experimental |

Abicht (2023) reviewed 95 reasoners; many are unusable or abandoned. The gap for **maintained, embeddable, open-source** reasoners is real — especially outside the JVM.

---

## Performance patterns (from published benchmarks)

### OWL EL

- ELK classifies SNOMED CT in **~5 seconds** on quad-core hardware (vs minutes for older EL reasoners).
- ELK wins OWL EL tracks at ORE competitions when DL reasoners are compared on EL corpora.
- **Implication:** OntoLogos EL engine (v0.5) must be measured against **ELK and whelk-rs**, not HermiT.

### OWL RL

- RDFox: sub-second materialization on million-triple workloads (commercial).
- **reasonable**: ~7× AllegroGraph, ~38× OWLRL on Brick model materialization benchmarks.
- **Implication:** OntoLogos RL (v0.4) should use **reasonable + OWLRL** as open references, RDFox as aspirational target.

### OWL DL

- ESWC 2023: **Konclude** and **HermiT** top-two on consistency/classification over BioPortal large ontologies; Konclude fewer errors overall; HermiT strong on consistency checking.
- Konclude wins most ORE DL categories; ELK wins EL-specific tracks.
- **Implication:** DL reference harness should use **Konclude CLI**, not HermiT-only. HermiT remains useful as a secondary cross-check where it still runs.

---

## Architectural patterns worth copying

### 1. MORe — modular black-box composition

[MORe](https://www.cs.ox.ac.uk/isg/tools/MORe/) (Oxford) classifies by:

1. Extracting an **EL module** classifiable by ELK (or RL fragment by RDFox).
2. Delegating the **residual signature** to a full DL reasoner (HermiT/Pellet).

Properties:

- Pay-as-you-go when EL ontologies gain a few expressive axioms.
- Reasoners are **black boxes** — no internal coupling.
- Initially **TBox-only** (ABox ignored for classification completeness).

**OntoLogos mapping:** v1.5 hybrid profiles; `Profile::Auto` should implement signature/module partitioning, not single-engine guesswork.

### 2. Konclude — coupled saturation + tableau

Konclude integrates completion-based saturation **into** the tableau calculus (Steigmiller et al.). When few axioms use disjunction, saturation does most work; tableau handles the rest.

**OntoLogos mapping:** v1.9 DL preview and 2.0 should target **hybrid coupling**, not a pure HermiT hypertableau port.

### 3. ELK — goal-directed saturation + incremental partitions

ELK uses Closure/Todo queues, parallel rule firing, and **partition-based incremental classification** without per-derivation bookkeeping (Kazakov et al., ISWC 2013).

**OntoLogos mapping:** v0.5 EL implementation and v0.7 incremental must follow this literature, not naive re-classify.

### 4. RDFox / reasonable — indexed forward chaining

RL engines compile rules and index by head predicate for matching.

**OntoLogos mapping:** validates `TripleIndex` + parallel `RlEngine` design in ROADMAP v0.4.

---

## Rust ecosystem positioning

| Project | Role | OntoLogos relationship |
|---------|------|------------------------|
| **horned-owl** | OWL parse + manipulate | **Ally** — v0.2 parser dependency |
| **whelk-rs** | OWL EL reasoner | **Peer** — EL conformance benchmark; study Kazakov rule implementation |
| **reasonable** | OWL RL reasoner | **Peer** — RL conformance benchmark |
| **py-horned-owl / py-whelk** | Python + horned-owl + whelk | **Ecosystem** — potential interoperability, not primary API |
| **fukurow** | WASM OWL DL + SPARQL | Different niche (browser/WASM) |
| **open-ontologies** | MCP + claimed DL tableaux | Unverified; monitor, do not design around |

Horned-OWL authors (TGDK 2024) explicitly note: **most OWL2 reasoners are abandoned**; whelk-rs covers EL only; **no Rust DL reasoner** in that ecosystem yet. OntoLogos 2.0 fills that gap if delivered with maintenance commitment.

---

## Strategic revelations for OntoLogos

### Opportunity

- JVM reasoner stagnation + Protégé’s reliance on legacy HermiT creates demand for **maintained, embeddable** alternatives.
- Biomedical pipelines (GO, SNOMED, OBO) are EL-heavy — ELK parity in Rust is high value even without DL.

### Risks

- **whelk-rs** and **reasonable** are ahead on EL/RL respectively; OntoLogos must ship competitive engines or justify value via unified API, CLI, Python, Ontocode.
- **Konclude** sets a high DL bar; full HermiT parity in 2.0 may be unrealistic — scope hybrid DL and document gaps.
- **MORe-style hybrid** is required for real ontologies that mix EL TBox with a few DL axioms.

### Plan changes (see PLAN.md)

1. Shift DL architectural reference from **HermiT-only** to **Konclude hybrid + MORe modular routing**.
2. Add **whelk-rs** and **reasonable** to benchmark conformance targets.
3. Mandate **ELK taxonomy extraction algorithm** (transitive reduction over equivalence classes) for v0.5.
4. Mandate **partition-based incremental EL** (Kazakov 2013) for v0.7.
5. Explicitly **do not depend on OWL API** — it anchors the JVM stack we are replacing.
6. Position Ontocode over Protégé plugin strategy given HermiT plugin stagnation.

---

## References

- Abicht, K. (2023). *OWL Reasoners still useable in 2023*. arXiv:2309.06888. Data: https://github.com/k00ni/owl-reasoner-list
- Kazakov, Y., Krötzsch, M., Simancik, F. (2014). *The Incredible ELK*. JAR 53(1).
- Kazakov, Y., et al. (2013). *Incremental Classification of EL+ Ontologies*. ISWC 2013.
- Armas Romero, A., et al. (2012). *MORe: Modular Combination of OWL Reasoners*. ISWC 2012.
- Steigmiller, A., et al. (2014). *Coupling Tableau Algorithms with Completion-based Saturation* (Konclude).
- ESWC 2023 DMKG workshop paper 2861 — six DL reasoners on BioPortal corpora.
- Lord, P., et al. (2024). *Horned-OWL: Flying Further and Faster with Ontologies*. TGDK 2(2).
