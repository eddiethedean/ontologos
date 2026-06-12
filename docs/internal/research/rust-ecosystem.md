# Rust OWL Ecosystem Study

Competitive and complementary Rust projects relevant to OntoLogos (reviewed 2026-06).

**Update (2026-06):** whelk and reasonable are **dependencies**, not just conformance peers. See [dependency-first ADR](../design/dependency-first.md).

## horned-owl (dependency — parsing)

- **Role:** OWL 2 parse, manipulate, serialize (RDF/XML, functional, etc.)
- **Repo:** https://github.com/phillord/horned-owl
- **OntoLogos:** `ontologos-parser` integration; EL bridge model for whelk

## whelk-rs (dependency — EL)

- **Role:** OWL 2 EL classifier (Rust port of Whelk / Kazakov rules)
- **Repo:** https://github.com/INCATools/whelk-rs
- **OntoLogos:** `ontologos-el` facade delegates to whelk via `ontologos-bridge`
- **Note:** Not on crates.io yet; pinned git rev in workspace

## reasonable (dependency — RL/RDFS)

- **Role:** OWL 2 RL forward chaining via DataFrog Datalog
- **Repo:** https://github.com/gtfierro/reasonable
- **OntoLogos:** `ontologos-rl` and `ontologos-rdfs` facades delegate to reasonable
- **License:** BSD-3-Clause (document in NOTICES)

## petgraph (dependency — graph algorithms)

- **Role:** Directed graphs, traversals, acyclic checks
- **OntoLogos:** `ontologos-query` hierarchy views; `ontologos-explain` proof graphs; taxonomy transitive reduction in bridge

## Differentiation summary

| Capability | whelk-rs | reasonable | OntoLogos |
|------------|----------|------------|-----------|
| EL classify | Yes (engine) | No | Facade + Taxonomy API |
| RL materialize | No | Yes (engine) | Facade + core model |
| Unified CLI/Python | Partial | Partial | **Yes** |
| Profile detection | No | No | **Yes** |
| JSON v2 embed model | No | No | **Yes** |

**Positioning:** OntoLogos is a **maintained orchestration stack** — not the first Rust reasoner, but the first aiming for unified EL + RL + RDFS + explanations under one semver-governed workspace built on whelk and reasonable.
