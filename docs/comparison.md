# Comparison with Existing Tools

Honest positioning for evaluators. **Read [Evaluator scope](guides/evaluator-scope.md) first** — HermiT parity metrics apply to **889 gated conformance cases**, not every real-world ontology.

**v1.1.1** covers stable EL, RL, RDFS, **OWL 2 DL**, and **DLSafe SWRL** on crates.io and PyPI.

OntoLogos passes the in-scope HermiT catalog gate (`parity_pct = 100%` on **889** cases) and the composite `true_parity_pct` gate at **100%** in blocking CI. Blocking CI runs **450** Java axiom + **428** OWL WG tests @ 30s.

Before production DL cutover, validate on **your** corpus. See [When not to use OntoLogos](guides/when-not-to-use.md) and [Release status](project/release-status.md).

## Maturity matrix

| Capability | OntoLogos (v1.1.1) | ELK | HermiT | Konclude | reasonable | whelk-rs | Protégé |
|------------|-------------------|-----|--------|----------|------------|----------|---------|
| Load OWL files | **Yes** (partial mapping) | Yes | Yes | Yes | Yes | Yes | Yes |
| OWL profile detection | **Yes** | No | No | No | No | No | Via plugin |
| OWL EL classification | **Yes** (in-house) | **Yes** | Slow/overkill | Yes | No | **Yes** | Via plugin |
| OWL RL reasoning | **Yes** (via reasonable) | No | Partial | Partial | **Yes** | No | Via plugin |
| RDFS materialization | **Yes** (via reasonable) | No | Yes | Yes | Partial | No | Yes |
| OWL DL (gated corpora) | **In-scope + true parity gates green** (`ontologos-dl`; 450+428 @ 30s) | No | Yes (stagnant) | Yes | No | No | Via plugin |
| OWL DL (PyPI / crates.io) | **Yes** (`profile="dl"`) | No | Yes (stagnant) | Yes | No | No | Via plugin |
| Embeddable Rust API | **Yes** | JVM only | JVM only | C++/OWLlink | **Yes** | **Yes** | Desktop IDE |
| Unified multi-profile CLI/Python | **Yes** | No | No | No | RL only | EL only | Via plugins |
| Maintained (2026) | **Active** | **Active** | Stagnant | **Active** | **Active** | **Active** | Active (editor) |
| Hybrid EL+DL routing | **Yes** | No | No | Internal | No | No | MORe plugin |
| Explanations | EL-first | Yes | Yes | Partial | Limited | No | Yes |
| Production-ready (EL/RL/RDFS) | **Yes** (within mapped construct subset) | Yes | Legacy | Yes | RL-focused | Experimental | Yes |
| Production-ready (OWL DL) | **Yes** on v1.0.0 (validate your corpus) | No | Yes (stagnant) | Yes | No | No | Via plugin |

CLI `classify --profile auto|el|rl|rdfs|alc|dl|dl-preview|swrl` routes via `ontologos-facade`. Preview profiles: [Profile stability matrix](guides/profile-stability.md). Use `materialize` for explicit RDFS.

## What OntoLogos adds over raw dependencies

| You need… | Use upstream directly | Use OntoLogos |
|-----------|----------------------|---------------|
| RL materialization only | `reasonable` crate or PyPI | Profile routing + core model + CLI |
| EL classification only | ELK or whelk-rs + horned-owl | Taxonomy API + query + JSON v3 + explain |
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
- CLI or Python batch workflows across EL, RL, and DL
- JVM-free HermiT replacement on gated corpora
- Contributing to a unified open-source Rust ontology stack

## When to use incumbents directly

See [When not to use OntoLogos](guides/when-not-to-use.md) for the full decision guide. Summary:

- **ELK / whelk-rs:** EL-only workflows; maximum EL performance tuning
- **reasonable:** RL-only; triple-store or incremental materialization without core model
- **Konclude:** maximum DL performance on very large ontologies
- **Protégé + HermiT/ELK:** interactive OWL editing
- **owlready2:** Python-centric workflows with JVM backends

## OntoLogos target (1.0 vs 2.0)

**1.0** delivers OWL DL HermiT parity on gated corpora (published **v1.1.1** on crates.io/PyPI). **2.0** extends beyond HermiT (Konclude-class performance, breaking API where needed).

Replace JVM-bound **batch** reasoning in Rust/Python pipelines via stable facade APIs, with CLI, Python, and Ontocode integration.

See [Roadmap summary](project/roadmap-summary.md).
