# Architecture Overview

OntoLogos is a Cargo workspace of profile-specific reasoning crates layered on a shared in-memory ontology model. This page is for adopters evaluating integration — see [SPEC.md](project/spec.md) for detailed API status tags.

## Crate dependency graph

```mermaid
flowchart TB
  core[ontologos_core]
  parser[ontologos_parser]
  profile[ontologos_profile]
  rdfs[ontologos_rdfs]
  rl[ontologos_rl]
  el[ontologos_el]
  query[ontologos_query]
  explain[ontologos_explain stub]
  cli[ontologos_cli]
  py[ontologos_py]
  conformance[ontologos_conformance]

  core --> parser
  parser --> profile
  core --> rdfs
  rdfs --> rl
  core --> el
  core --> query
  core --> explain
  parser --> cli
  profile --> cli
  rdfs --> cli
  el --> cli
  rl --> cli
  explain --> cli
  parser --> py
  rdfs --> py
  rl --> py
  el --> py
  rl --> conformance
  rdfs --> conformance
  el --> conformance
  parser --> conformance
```

Published to crates.io (v0.5): `ontologos-core`, `ontologos-parser`, `ontologos-profile`, `ontologos-rdfs`, `ontologos-rl`, `ontologos-el`, `ontologos-query`.

Workspace-only: `ontologos-cli`, `ontologos-conformance`, `ontologos-explain` (stub), `ontologos-py` (also on PyPI).

## Data flow

```mermaid
flowchart LR
  subgraph input [Input]
    builder[OntologyBuilder]
    json[JSON_v2]
    owl[OWL_RDF_files]
  end

  subgraph model [CoreModel]
    ontology[Ontology]
  end

  subgraph engines [Engines]
    rdfsEng[RdfsEngine]
    rlEng[RlEngine]
    elEng[ElClassifier]
  end

  builder --> ontology
  json --> ontology
  owl --> parser[load_ontology] --> ontology
  ontology --> profileDet[detect_profile]
  ontology --> rdfsEng
  rdfsEng --> rlEng
  ontology --> elEng
```

1. **Construct or load** an `Ontology` (builder, JSON, or parser).
2. **Optionally detect profile** with `ontologos_profile::detect_profile`.
3. **Run an engine** that mutates the ontology in place (RDFS/RL) or returns a `Taxonomy` (EL).
4. **Query** via `ontologos-query` or indexes on `Ontology`.

## Core model (`ontologos-core`)

Single in-memory representation:

| Component | Role |
|-----------|------|
| `InternPool` | Deduplicated IRI strings |
| `EntityRegistry` | Typed entities (class, individual, properties) |
| `AxiomStore` | Structured TBox and ABox axioms |
| `AxiomIndex` | Secondary indexes for traversal |
| `ParseMeta` | Parser scan metadata (optional) |
| `Taxonomy` | EL classification output (subsumptions, equivalences, unsatisfiable) |

Serialization: JSON snapshot v2 (`to_json` / `from_json`).

**Deliberate split:** `Ontology::from_file` returns `ParseNotAvailable` so `ontologos-core` has no parser dependency. File loading lives in `ontologos-parser`.

## Reasoner facade

`Reasoner` in core is a configuration wrapper around `Ontology`:

| `Profile` | `Reasoner::classify()` in core | Use instead |
|-----------|-------------------------------|-------------|
| `Auto` | `NotImplemented` | `ontologos_el::classify_with_profile` |
| `El` | `NotImplemented` | `ontologos_el::classify_reasoner` or `ElClassifier` |
| `Rdfs` | Delegate hint (`Error::Message`) | `ontologos_rdfs::materialize_reasoner` |
| `Rl` | Delegate hint | `ontologos_rl::classify_reasoner` |

CLI and Python call `ontologos_el::classify_with_profile` for routed classification.

## Engine layering

| Engine | Crate | Scope |
|--------|-------|-------|
| RDFS | `ontologos-rdfs` | TBox: `subClassOf`/`subPropertyOf` closure, domain/range inheritance |
| OWL RL | `ontologos-rl` | RDFS pass + RL TBox/ABox rules until saturation |
| OWL EL | `ontologos-el` | Completion-based taxonomy (`ElClassifier`) |
| Query | `ontologos-query` | Taxonomy queries (`is_subsumed`, `direct_subclasses`, …) |

RL always runs RDFS first inside `RlEngine::saturate`.

## CLI surface

`ontologos-cli` wires parser, profile detection, RDFS, RL, and EL:

- `profile` — detect OWL profile
- `classify --profile auto|el|rl|rdfs` — routed classification
- `materialize` — explicit RDFS materialization

## Design choices

| Choice | Rationale |
|--------|-----------|
| No umbrella `ontologos` crate | Depend only on what you need; smaller dependency trees |
| Core/parser split | Embed data model without OWL parse stack |
| In-place materialization | RDFS/RL engines add axioms to the same `Ontology` |
| EL taxonomy overlay | EL classification returns `Taxonomy` without mutating asserted axioms |
| Partial OWL mapping | Map named TBox/ABox shapes; scan rest for profile diagnostics |
| Batch fixed-point engines | Saturation loops until no new axioms (not incremental yet) |

## Related

- [Choosing an API](guides/choosing-an-api.md)
- [Supported constructs](reference/supported-constructs.md)
- [SPEC.md](project/spec.md)
- [Roadmap summary](project/roadmap-summary.md)
