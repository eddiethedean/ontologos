# HermiT → OntoLogos Replacement Matrix

> **Status:** Living document (2026-06-12). HermiT source is a **local reference checkout** at `HermiT/` (gitignored). Ported tests live in `crates/ontologos-conformance` with manifest `tests/hermit/manifest.toml`.

## Strategy

OntoLogos does **not** plan a line-by-line HermiT hypertableau port. Replacement is **profile-modular**:

| User need today | HermiT | OntoLogos path |
|-----------------|--------|----------------|
| Protégé DL classification | HermiT (unmaintained) | **1.0** `ontologos-dl` (Konclude-style hybrid); HermiT JAR as conformance cross-check |
| Biomedical EL batch | Often ELK, not HermiT | **0.5** `ontologos-el` (ELK-style completion) |
| Rule materialization | Partial via RL rules | **0.4** `ontologos-rl` + **0.3** RDFS |
| Embeddable Rust / Python | JVM only | **0.3+** crates + **0.9** PyPI |
| Explanations | HermiT justifications | **0.6** `ontologos-explain` |
| Incremental edits | HermiT buffered reasoner | **0.7** dependency-indexed re-classify |

**1.0 goal:** **Full HermiT parity** — replace HermiT for OWL 2 DL batch classification, explanation, and Rust/Python/CLI workflows (HermiT conformance Tiers A–C).  
**2.0 goal:** **Beyond HermiT** — Konclude-class performance, breaking API evolution, and features outside HermiT's surface.

## HermiT capability map

| HermiT component | Java package | OntoLogos target | Milestone |
|------------------|--------------|----------------|-----------|
| OWL API load + normalize | `structural` | `ontologos-parser` + normal forms in engines | 0.2 / 0.5 |
| Clausification (DL clauses) | `structural` | `ontologos-dl` internal | 2.0 |
| Tableau / hypertableau | `tableau` | `ontologos-dl` | 2.0 |
| EL-style completion | (via tableau) | `ontologos-el` (ELK algorithm) | 0.5 |
| OWL RL datalog rules | `reasoner.RulesTest` | `ontologos-rl` | 0.4 |
| RDFS / subClassOf closure | (DL over TBox) | `ontologos-rdfs` | **0.3** |
| Class taxonomy | `Reasoner.printHierarchies` | `ontologos-el::Taxonomy` + query | 0.5 |
| ABox consistency | `isConsistent` | DL tableau + RL/EL fragments | 1.6+ |
| Entailment checker | `EntailmentChecker` | `ontologos-query` + explain | 0.6 / 1.8 |
| Explanations | `debugger` | `ontologos-explain` | 0.6 |
| Incremental flush | buffered reasoner | core `AxiomId` dependency index | 0.7 |
| OWL WG test suite | `owl_wg_tests` | conformance harness | 1.0+ |
| OWLlink eval tests | `reasoner.OWLLinkTest` | conformance harness | 0.4–2.0 |
| Datatype reasoning | `DatatypesTest`, … | DL engine + literal index | 2.0 |
| SWRL / DLSafe rules | `RulesTest` | `ontologos-swrl` | 1.0 |

## OWL API surface (Protégé parity)

| OWLReasoner method (typical) | HermiT | OntoLogos |
|------------------------------|--------|-----------|
| `getSubClasses` | Yes | 0.5 query over taxonomy |
| `isEntailed` | Yes | 0.6 explain + 1.8 query |
| `isConsistent` | Yes | 2.0 DL / partial EL |
| `getEquivalentClasses` | Yes | 0.5 EL |
| `getUnsatisfiableClasses` | Yes | 0.5 EL / 2.0 DL |
| `precomputeInferences` | Yes | 0.7 incremental |
| `flush` / buffered changes | Yes | 0.7 |

## HermiT test inventory → port plan

HermiT ships **59** Java test classes under `project/test/`. Grouped by port tier:

### Tier A — run in CI without HermiT checkout

Logic ported inline; provenance tracked in `tests/hermit/manifest.toml`. **23** Rust tests (6 RDFS + 17 RL) as of v0.4.

| HermiT test | OntoLogos port | Engine |
|-------------|----------------|--------|
| `ReasonerTest.testSubsumption1` | `hermit_rdfs::subsumption1_transitive_subclass` | RDFS |
| `ReasonerTest.testSubAndSuperConcepts` | `hermit_rdfs::sub_and_super_concepts` | RDFS |
| `ReasonerTest.testSubAndSuperRoles` | `hermit_rdfs::sub_and_super_roles` | RDFS |
| `OWLLinkTest` update hierarchy (buffered/non-buffered) | `hermit_rdfs::owllink_update_hierarchy_*` | RDFS |
| `ReasonerTest.testSubsumption2/3` | `hermit_rl::subsumption2_*`, `subsumption3_*` | RL |
| `ReasonerTest.testSameAs` | `hermit_rl::same_as_propagates_class_assertion` | RL |
| `ReasonerTest.testEquivalentClassInstances` | `hermit_rl::equivalent_class_instances_share_types` | RL |
| `ReasonerTest.testReflexiveAndSameAs` | `hermit_rl::reflexive_and_same_as_expand_property_instances` | RL |
| `ReasonerTest.testIndividualRetrievalBug` | `hermit_rl::individual_property_retrieval` | RL |
| `ReasonerTest.testIsFunctionalObject` | `hermit_rl::functional_property_characteristic_propagates_to_subproperty` | RL |
| `ReasonerTest.testIsAsymmetricObject` | `hermit_rl::asymmetric_property_characteristic_propagates_to_subproperty` | RL |
| RL fragment coverage (no HermiT method) | `hermit_rl::{property_assertion_*, inverse_*, symmetric_*, transitive_*, domain_*, range_*, equivalent_*, disjoint_*}` | RL |

**Tier A excluded** (documented in manifest, not ported):

| HermiT test | Reason |
|-------------|--------|
| `ReasonerTest.testSubProperties`, `testObjectPropertyHierarchy` | `SubObjectPropertyOf` with inverse operands — parser not mapped |
| `ReasonerTest.testIsSymmetricObject`, `testIsTransitiveObject` | HermiT does not propagate symmetric/transitive to subproperties; OntoLogos RL does (OWL RL prp-fp/ap) |
| Incremental/buffered reasoner tests | No buffered reasoner until v0.7 |
| Nominals, complements, role chains, hasKey | DL / unmapped constructs |

### Tier B — optional local (`HermiT/` present)

| HermiT suite | Count | Milestone | Notes |
|--------------|-------|-----------|-------|
| `reasoner/res/OWLLink/*.owl` load | ~40 files | 0.2 | Parser smoke (UTF-8; ISO-8859-1 RDF/XML gap) |
| `ClassificationTest` (pizza, wine, galen) | 4 | 0.5 | Taxonomy golden files |
| `owl_wg_tests` entailment | large | 1.0 | Approved WG subset |
| `structural/ClausificationTest` | 8+ | 2.0 | Internal DL clauses |
| `tableau/*` | 10+ | 2.0 | Engine internals |
| Datatype / literal tests | 8 | 2.0 | XSD facets |
| `RulesTest` (SWRL) | 15+ | — | Not planned 1.x |

### Tier C — reference only (run HermiT JAR for baseline)

Heavy classification corpora (GALEN full, OWLlink Bob tests). Use `benchmarks/` + Konclude/HermiT harness at **1.9/2.0**.

## Conformance workflow

1. Clone HermiT beside OntoLogos (already at `HermiT/`, gitignored).
2. Run `cargo test -p ontologos-conformance` — Tier A always; Tier B skipped if `HermiT/` missing.
3. Run `cargo test -p ontologos-conformance -- --ignored` locally with full HermiT tree.
4. Add manifest entries when porting a Java test; link `source_class` + `source_method`.

Environment override: `ONTOLOGOS_HERMIT_ROOT=/path/to/HermiT`

## Exit criteria vs HermiT

| Release | HermiT replacement claim |
|---------|--------------------------|
| **0.3** | RDFS TBox materialization; not a HermiT substitute |
| **0.5** | EL classification: parity with ELK/whelk-rs on Pizza, GO-subset (not HermiT) |
| **0.4** | RL materialization: parity with reasonable on Family |
| **1.0** | Full stack for EL/RL/RDFS batch; HermiT only for DL spot checks |
| **2.0** | DL classification within 10× Konclude on standard corpora; HermiT secondary agreement |

## Related

- [hermit.md](hermit.md) — architecture study
- [konclude.md](konclude.md) — 2.0 DL north star
- [tests/hermit/manifest.toml](../../../tests/hermit/manifest.toml) — ported test catalog
- [ROADMAP.md](../../../ROADMAP.md) — release milestones
