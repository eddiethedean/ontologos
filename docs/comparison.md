# Comparison with Existing Tools

Honest positioning for evaluators. OntoLogos is **not** a drop-in HermiT replacement. From v0.6 onward it **orchestrates** `whelk` (EL) and `reasonable` (RL/RDFS) behind a unified API.

See [landscape-2023.md](https://github.com/eddiethedean/ontologos/blob/main/docs/internal/research/landscape-2023.md) for the full reasoner survey.

## Maturity matrix

| Capability | OntoLogos v0.6+ | ELK | HermiT | Konclude | reasonable | whelk-rs | Protégé |
|------------|-----------------|-----|--------|----------|------------|----------|---------|
| Load OWL files | **Yes** (partial mapping) | Yes | Yes | Yes | Yes | Yes | Yes |
| OWL profile detection | **Yes** | No | No | No | No | No | Via plugin |
| OWL EL classification | **Yes** (via whelk) | **Yes** | Slow/overkill | Yes | No | **Yes** | Via plugin |
| OWL RL reasoning | **Yes** (via reasonable) | No | Partial | Partial | **Yes** | No | Via plugin |
| RDFS materialization | **Yes** (via reasonable) | No | Yes | Yes | Partial | No | Yes |
| OWL DL | No (2.0) | No | Yes (stagnant) | **Yes** | No | No | Via plugin |
| Embeddable Rust API | **Yes** | JVM only | JVM only | C++/OWLlink | **Yes** | **Yes** | Desktop IDE |
| Unified multi-profile CLI/Python | **Yes** | No | No | No | RL only | EL only | Via plugins |
| Maintained (2026) | **Active** | **Active** | Stagnant | **Active** | **Active** | **Active** | Active (editor) |
| Hybrid EL+DL routing | Planned (v1.5) | No | No | Internal | No | No | MORe plugin |
| Explanations | EL-first (v0.6+) | Yes | Yes | Partial | Limited | No | Yes |
| Production-ready | **Pre-release** | Yes | Legacy | Yes | RL-focused | Experimental | Yes |

CLI `classify --profile auto|el|rl|rdfs` routes to whelk (EL), reasonable (RL/RDFS). Use `materialize` for explicit RDFS.

## What OntoLogos adds over raw dependencies

| You need… | Use upstream directly | Use OntoLogos |
|-----------|----------------------|---------------|
| RL materialization only | `reasonable` crate or PyPI | Profile routing + core model + CLI |
| EL classification only | `whelk` + horned-owl | Taxonomy API + query + JSON v2 |
| Parse OWL safely | horned-owl + your limits | `ontologos-parser` with `ParseLimits` |
| One CLI for all profiles | Multiple tools | `ontologos classify --profile auto` |
| Python batch pipeline | `reasonable` / `py-whelk` separately | `pip install ontologos` unified facade |

## Rust dependencies (not competitors)

| Project | Role in OntoLogos |
|---------|-------------------|
| **horned-owl** | Parsing and EL bridge model |
| **whelk-rs** | OWL EL engine (git dependency) |
| **reasonable** | OWL RL and RDFS engine |
| **petgraph** | Taxonomy and proof-graph algorithms |

OntoLogos targets a **maintained orchestration stack** with MORe-style hybrid routing (v1.5), not reimplementing EL/RL rule engines.

## When to use OntoLogos

- Embedding an ontology **data model** in Rust with profile routing
- Loading OWL files, detecting profiles, and classifying in one workspace
- CLI or Python batch workflows across EL and RL
- Contributing to a unified open-source Rust ontology stack

## When to use incumbents directly

- **ELK / whelk-rs:** EL-only workflows; maximum EL performance tuning
- **reasonable:** RL-only; triple-store or incremental materialization without core model
- **Konclude:** full DL batch reasoning today
- **Protégé + HermiT/ELK:** interactive OWL editing
- **owlready2:** Python-centric workflows with JVM backends

## OntoLogos target (1.0)

Replace JVM-bound **batch** reasoning in Rust/Python pipelines via facade APIs over whelk + reasonable, with CLI, Python, and Ontocode integration. Full OWL DL in 2.0 extends the whelk/horned-owl kernel rather than a greenfield rewrite.

See [Roadmap summary](project/roadmap-summary.md) and [dependency-first ADR](internal/design/dependency-first.md).
