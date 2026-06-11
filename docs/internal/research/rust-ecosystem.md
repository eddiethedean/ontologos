# Rust OWL Ecosystem Study

Competitive and complementary Rust projects relevant to OntoLogos (reviewed 2026-06).

## horned-owl (ally — parsing)

- **Role:** OWL 2 parse, manipulate, serialize (RDF/XML, functional, etc.)
- **Repo:** https://github.com/phillord/horned-owl
- **Maturity:** Mature library; full OWL 2 + SWRL; TGDK 2024 artifact
- **OntoLogos:** v0.2 parser integration (already planned)

Authors note that **no DL reasoner** exists in the Horned-OWL ecosystem yet; whelk-rs covers EL only. OntoLogos reasoning crates complement horned-owl's manipulation layer.

## whelk-rs (peer — EL)

- **Role:** OWL 2 EL classifier (Rust port of Whelk / Kazakov rules)
- **Repo:** https://github.com/INCATools/whelk-rs
- **Maintainer:** INCATools / James Balhoff ecosystem (OBO, Uberon)
- **Status:** Experimental but actively developed; `py-whelk` on PyPI
- **Gaps:** Consistency checking not implemented (per py-whelk docs)

**Implications:**

- Primary **Rust EL conformance target** alongside ELK (Java).
- Study for v0.5 rule implementation; do not duplicate horned-owl's OWL model — OntoLogos uses its own `Axiom` store.
- OBO/Uberon community may already adopt whelk — OntoLogos should interoperate via OWL files, not require migration.

## reasonable (peer — RL)

- **Role:** OWL 2 RL forward chaining via DataFrog Datalog
- **Repo:** https://github.com/gtfierro/reasonable
- **Status:** Actively maintained (releases 2026); Rust lib + CLI + Python
- **Performance:** Published wins vs OWLRL and AllegroGraph on Brick models

**Implications:**

- Primary **open RL conformance target** for v0.4 (with OWLRL as secondary).
- Different internal model (RDF triples + Datalog) — compare outputs, not internals.
- Python bindings exist — OntoLogos PyPI package must document when to use `reasonable` vs `ontologos`.

## fukurow (adjacent — WASM DL)

- **Role:** WASM-native OWL Lite/DL + SPARQL + SHACL
- **Repo:** https://github.com/com-junkawasaki/fukurow
- **Niche:** Browser/cyber-defense; full stack in one project

Not a direct competitor for server-side batch reasoning; monitor for WASM export ideas (out of 1.x scope).

## open-ontologies (adjacent — MCP)

- **Role:** MCP server, Oxigraph store, claimed OWL2-DL tableaux + EL
- **Status:** Very new; broad scope (70+ MCP tools)

Treat as unverified until independently benchmarked; do not pivot plan toward MCP-first design.

## Differentiation summary

| Capability | whelk-rs | reasonable | OntoLogos target |
|------------|----------|------------|------------------|
| EL classify | Yes | No | v0.5 |
| RL materialize | No | Yes | v0.4 |
| RDFS | No | Partial (via RL rules) | v0.3 |
| DL | No | No | 2.0 |
| Unified API | No | RL only | Yes |
| CLI + Python + IDE path | Partial | Partial | Yes (1.0) |
| Modular profiles / hybrid | No | No | v1.5 (MORe-style) |

**Positioning:** OntoLogos is a **maintained, modular, multi-profile stack** — not the first Rust reasoner, but the first aiming for EL + RL + RDFS + DL + explanations under one semver-governed workspace.
