# Comparison with Existing Tools

Honest positioning for evaluators. OntoLogos v0.4 is **not** a drop-in HermiT replacement.

See [landscape-2023.md](internal/research/landscape-2023.md) for the full 2023–2026 reasoner survey.

## Maturity matrix

| Capability | OntoLogos v0.4 | ELK | HermiT | Konclude | reasonable | whelk-rs | Protégé |
|------------|----------------|-----|--------|----------|------------|----------|---------|
| Load OWL files | **Yes** (partial mapping) | Yes | Yes | Yes | Yes | Yes | Yes |
| OWL profile detection | **Yes** | No | No | No | No | No | Via plugin |
| OWL EL classification | No (v0.5) | **Yes** | Slow/overkill | Yes | No | **Yes** | Via plugin |
| OWL RL reasoning | **Yes** (library/Python) | No | Partial | Partial | **Yes** | No | Via plugin |
| RDFS materialization | Yes (TBox) | No | Yes | Yes | Partial | No | Yes |
| OWL DL | No (2.0) | No | Yes (stagnant) | **Yes** | No | No | Via plugin |
| Embeddable Rust API | **Yes** | JVM only | JVM only | C++/OWLlink | **Yes** | **Yes** | Desktop IDE |
| Maintained (2026) | **Active** | **Active** | Stagnant | **Active** | **Active** | **Active** | Active (editor) |
| Hybrid EL+DL routing | No (v1.5) | No | No | Internal | No | No | MORe plugin |
| Explanations | No (v0.6) | Yes | Yes | Partial | Limited | No | Yes |
| Production-ready | **Pre-release** | Yes | Legacy | Yes | RL-focused | Experimental | Yes |

CLI `classify` and `materialize` remain RDFS-only until v0.5; OWL RL saturation is available via `ontologos-rl` (Rust library) or Python `profile="rl"`.

v0.4 maps a **subset** of TBox and named ABox axioms into core; complex expressions are scanned for profile diagnostics but not stored. See [supported-constructs.md](reference/supported-constructs.md).

## Maintenance landscape

Most JVM DL reasoners (HermiT, Pellet, FaCT++) have had **little or no development since ~2015–2020** ([Abicht 2023](https://arxiv.org/abs/2309.06888)). ELK and Konclude remain the maintained performance leaders for EL and DL respectively. This is a core motivation for OntoLogos.

## Rust alternatives

| Project | Strength | OntoLogos difference |
|---------|----------|----------------------|
| **whelk-rs** | OWL EL in Rust | EL only; no RL/DL/CLI/Python stack |
| **reasonable** | OWL RL in Rust | RL only; triple/Datalog model |
| **horned-owl** | OWL parse/manipulate | No reasoner; OntoLogos uses it for v0.2 parsing |

OntoLogos targets a **unified multi-profile workspace** with MORe-style hybrid routing (v1.5), not a single-profile library.

## When to use OntoLogos today

- Embedding an ontology **data model** in Rust
- Loading OWL files and detecting profiles natively
- OWL RL saturation via `ontologos-rl` or Python `profile="rl"`
- Evaluating the architecture and roadmap
- Contributing to a maintained open-source Rust reasoner stack

## When to use incumbents

- **ELK / whelk-rs:** EL classification today (SNOMED, GO, OBO)
- **reasonable:** RL materialization in Rust/Python today (mature triple-store model)
- **Konclude:** full DL batch reasoning today (CLI/OWLlink)
- **Protégé + HermiT/ELK:** interactive OWL editing (note HermiT maintenance risk)
- **owlready2:** Python-centric OWL workflows with JVM reasoning backends
- **RDFox:** high-performance commercial RL/DLS materialization

## OntoLogos target (1.0)

Replace JVM-bound **batch** reasoning in Rust/Python pipelines with native EL/RL/RDFS engines, CLI, and IDE integration (Ontocode). Conformance measured against **ELK + whelk-rs** (EL) and **reasonable** (RL). Full OWL DL in 2.0, architected after **Konclude** hybrid design rather than a legacy HermiT port.

See [ROADMAP.md](../ROADMAP.md) for milestone sequencing and exit criteria.

For HermiT-ported test coverage and known gaps, see [Conformance coverage](reference/conformance.md).
