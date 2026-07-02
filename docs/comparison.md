# Comparison with Existing Tools

Honest positioning for evaluators. **Published v0.9.0** covers stable EL, RL, and RDFS.

**`main` (1.0.0 workspace, not yet tagged on PyPI)** passes the in-scope HermiT catalog gate (`parity_pct = 100%` on **889** cases) and the composite `true_parity_pct` gate at **100%** in blocking CI. Blocking CI runs **450** Java axiom + **428** OWL WG tests @ 30s.

These metrics apply only to the **gated conformance corpora** — not every real-world ontology. See [Evaluator scope](guides/evaluator-scope.md) and [Release status](project/release-status.md).

See [landscape-2023.md](https://github.com/eddiethedean/ontologos/blob/main/docs/internal/research/landscape-2023.md) for the full reasoner survey.

## Maturity matrix

| Capability | OntoLogos (v0.9.0 / `main`) | ELK | HermiT | Konclude | reasonable | whelk-rs | Protégé |
|------------|-----------------|-----|--------|----------|------------|----------|---------|
| Load OWL files | **Yes** (partial mapping) | Yes | Yes | Yes | Yes | Yes | Yes |
| OWL profile detection | **Yes** | No | No | No | No | No | Via plugin |
| OWL EL classification | **Yes** (in-house) | **Yes** | Slow/overkill | Yes | No | **Yes** | Via plugin |
| OWL RL reasoning | **Yes** (via reasonable) | No | Partial | Partial | **Yes** | No | Via plugin |
| RDFS materialization | **Yes** (via reasonable) | No | Yes | Yes | Partial | No | Yes |
| OWL DL (workspace / gated corpora) | **In-scope + true parity gates green** (`ontologos-dl` on `main`; 450+428 @ 30s) | No | Yes (stagnant) | Yes | No | No | Via plugin |
| OWL DL (PyPI / crates.io today) | **v1.0.0 not yet published** — build from `main` for DL | No | Yes (stagnant) | Yes | No | No | Via plugin |
| Embeddable Rust API | **Yes** | JVM only | JVM only | C++/OWLlink | **Yes** | **Yes** | Desktop IDE |
| Unified multi-profile CLI/Python | **Yes** | No | No | No | RL only | EL only | Via plugins |
| Maintained (2026) | **Active** | **Active** | Stagnant | **Active** | **Active** | **Active** | Active (editor) |
| Hybrid EL+DL routing | **Pre-release** (`main`) | No | No | Internal | No | No | MORe plugin |
| Explanations | EL-first (v0.9.0+) | Yes | Yes | Partial | Limited | No | Yes |
| Production-ready | **Pre-release** | Yes | Legacy | Yes | RL-focused | Experimental | Yes |

CLI `classify --profile auto|el|rl|rdfs|alc|dl|dl-preview|swrl` routes via `ontologos-facade`. DL/ALC/SWRL status: [Profile stability matrix](guides/profile-stability.md). Use `materialize` for explicit RDFS.

## What OntoLogos adds over raw dependencies

| You need… | Use upstream directly | Use OntoLogos |
|-----------|----------------------|---------------|
| RL materialization only | `reasonable` crate or PyPI | Profile routing + core model + CLI |
| EL classification only | ELK or whelk-rs + horned-owl | Taxonomy API + query + JSON v2 + explain |
| Parse OWL safely | horned-owl + your limits | `ontologos-parser` with `ParseLimits` |
| One CLI for all profiles | Multiple tools | `ontologos classify --profile auto` |
| Python batch pipeline | `reasonable` / `py-whelk` separately | `pip install ontologos` unified facade |

## Rust dependencies (not competitors)

| Project | Role in OntoLogos |
|---------|-------------------|
| **horned-owl** | Parsing (via `ontologos-bridge`) |
| **reasonable** | OWL RL and RDFS engine |
| **petgraph** | Taxonomy and proof-graph algorithms |
| **whelk-rs** | Ecosystem peer for EL conformance benchmarks only (not a runtime dependency) |

OntoLogos targets a **maintained orchestration stack** with MORe-style hybrid routing (v1.5), not reimplementing RL rule engines.

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

## OntoLogos target (1.0 vs 2.0)

**1.0** delivers OWL DL HermiT parity on gated corpora (**both parity gates green on `main`**; crates.io/PyPI **v1.0.0** publish not yet shipped). **2.0** extends beyond HermiT (Konclude-class performance, breaking API where needed).

Replace JVM-bound **batch** reasoning in Rust/Python pipelines via stable facade APIs, with CLI, Python, and Ontocode integration.

See [Roadmap summary](project/roadmap-summary.md) and [dependency-first ADR](internal/design/dependency-first.md).
