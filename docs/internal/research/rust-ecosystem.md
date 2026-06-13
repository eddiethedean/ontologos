# Rust OWL Ecosystem Study

Competitive and complementary Rust projects relevant to OntoLogos (reviewed 2026-06).

**Update (2026-06):** reasonable is a **runtime dependency** for RL/RDFS. whelk-rs is an **ecosystem peer** for EL conformance benchmarks only (not a runtime dependency since v0.6.1). See [dependency-first ADR](../design/dependency-first.md).

## horned-owl (dependency — parsing)

- **Role:** OWL 2 parse, manipulate, serialize (RDF/XML, functional, etc.)
- **Repo:** https://github.com/phillord/horned-owl
- **OntoLogos:** `ontologos-parser` integration via `ontologos-bridge`

## whelk-rs (ecosystem peer — EL)

- **Role:** OWL 2 EL classifier (Rust port of Whelk / Kazakov rules)
- **Repo:** https://github.com/INCATools/whelk-rs
- **OntoLogos:** conformance peer; `ontologos-el` uses in-house completion (v0.6.1+)

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
| EL classify | Yes (peer) | No | In-house engine + Taxonomy API |
| RL materialize | No | Yes (engine) | Facade + core model |
| Unified CLI/Python | Partial | Partial | **Yes** |
| Profile detection | No | No | **Yes** |
| JSON v2 embed model | No | No | **Yes** |

**Positioning:** OntoLogos is a **maintained orchestration stack** — not the first Rust reasoner, but the first aiming for unified EL + RL + RDFS + explanations under one semver-governed workspace with in-house EL and reasonable RL/RDFS.
