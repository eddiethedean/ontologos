# OntoLogos Documentation

Native Rust ontology reasoning: load OWL files, detect profiles, run RDFS materialization and OWL RL saturation.

!!! warning "HermiT parity in progress (v1.0.0 workspace)"
    OntoLogos maps a **subset** of OWL axioms into its core model — `axiom_count()` is mapper output, not Protégé's total. CLI **`classify --profile auto`** routes to EL, RL, or DL (preview). Use **`materialize`** for explicit RDFS. **`explain`** builds proof graphs (EL full; RL/RDFS asserted-only). Full OWL DL HermiT parity is planned for **1.0**. See [Supported constructs](reference/supported-constructs.md).

!!! tip "Integration DO / DON'T"
    **DO** use CLI `ontologos classify`, Python `Reasoner(path=...).classify()`, or `ontologos_facade::classify` / profile crates (`ElClassifier`, `RlEngine`, `RdfsEngine`).

    **DON'T** call `ontologos_core::Reasoner::classify()` directly — it is a stub (`NotImplemented` or delegate hints). See [Choosing an API](guides/choosing-an-api.md).

!!! note "DL preview (main branch)"
    `--profile dl` and `--profile dl-preview` are available in CLI/Python for early testing. Preview mode may return `PreviewLimit` or `ResourceLimit`. Not suitable for production DL workflows. See [Preview profiles](guides/preview-profiles.md).

## Persona quick links

| I want to… | Start here |
|------------|------------|
| Try it in 5 minutes (no clone) | [Crates.io quick start](getting-started/index.md#cratesio-only-no-clone) |
| Try it from a clone | [Getting started index](getting-started/index.md) |
| Evaluate in 30 minutes | [Evaluator playbook](guides/evaluator-playbook.md) |
| Evaluate vs ELK / reasonable | [Comparison](comparison.md) |
| Integrate in Rust | [Choosing an API](guides/choosing-an-api.md) |
| Use Python | [Python guide](guides/python.md) |
| Contribute | [Contributing](project/contributing.md) |

## Capability matrix (v1.0.0)

| Capability | Library | CLI | Python |
|------------|---------|-----|--------|
| Load OWL files | Yes | Yes | Yes |
| In-memory ontology | Yes (`OntologyBuilder`) | No | Yes |
| Profile detection | Yes | `profile` | Via `"auto"` |
| RDFS materialization | Yes | `materialize` / `classify --profile rdfs` | `profile="rdfs"` |
| OWL RL saturation | Yes | `classify --profile rl` | `profile="rl"` |
| OWL EL taxonomy | Yes | `classify --profile el` | `profile="el"` |
| OWL DL (preview) | Yes (`ontologos-dl`) | `classify --profile dl\|dl-preview` | `profile="dl"` / `"dl-preview"` |
| Incremental reasoning | Yes (`ReasonerConfig::incremental`) | `--incremental` (session; multi-pass library) | `incremental=True` + mutation methods |
| Taxonomy queries | Yes (`ontologos-query`) | JSON output | `taxonomy` property |
| Explanations (EL full; RL/RDFS asserted-only) | Yes (`ontologos-explain`) | `explain` | `explain()` |
| Materialization reports | Yes | Yes (RDFS/RL) | Yes |
| Export saturated ontology | Yes (in-process) | No | No |
| Taxonomy DataFrame export | No | No | Yes (optional pandas/polars) |

## Learning path

1. **[First ontology](getting-started/first-ontology.md)** — builder API walkthrough
2. **[Load an OWL file](getting-started/load-owl-file.md)** — parser, formats, `ParseMeta`
3. **[RDFS materialization](getting-started/rdfs-materialization.md)** — TBox closure and domain/range
4. **[OWL RL saturation](getting-started/owl-rl-saturation.md)** — forward-chaining, reports, clashes
5. **[OWL EL classification](getting-started/owl-el-classification.md)** — completion-based taxonomy
6. **[Profile detection](guides/profile-detection.md)** — EL/RL/QL/DL + hybrid diagnostics
7. **[JSON snapshots](json-snapshot-v2.md)** — load and save ontologies
8. **[Error reference](reference/errors.md)** — core, parser, profile, engine errors
9. **[Roadmap summary](project/roadmap-summary.md)** — what ships next

## Getting started

| Document | Description |
|----------|-------------|
| [Getting started overview](getting-started/index.md) | Success paths by persona |
| [first-ontology.md](getting-started/first-ontology.md) | Build a taxonomy with `OntologyBuilder` |
| [load-owl-file.md](getting-started/load-owl-file.md) | Load OWL/RDF files with `ontologos-parser` |
| [rdfs-materialization.md](getting-started/rdfs-materialization.md) | RDFS TBox materialization |
| [owl-rl-saturation.md](getting-started/owl-rl-saturation.md) | OWL RL forward-chaining |
| [owl-el-classification.md](getting-started/owl-el-classification.md) | OWL EL taxonomy classification |
| [json-snapshot-v2.md](json-snapshot-v2.md) | JSON v2 format, limits, migration |

## Guides

| Document | Description |
|----------|-------------|
| [choosing-an-api.md](guides/choosing-an-api.md) | Which crate and entry point to use |
| [facade-api.md](guides/facade-api.md) | Unified `ontologos-facade::classify` routing |
| [preview-profiles.md](guides/preview-profiles.md) | DL, ALC, SWRL preview status and limits |
| [evaluator-playbook.md](guides/evaluator-playbook.md) | 30-minute Pizza/Family evaluation |
| [profile-detection.md](guides/profile-detection.md) | OWL profile detection and diagnostics |
| [python.md](guides/python.md) | Python bindings (`pip install ontologos`) |
| [glossary.md](guides/glossary.md) | OWL and OntoLogos terminology |
| [incremental-reasoning.md](guides/incremental-reasoning.md) | Incremental EL/RL/RDFS session API |
| [performance.md](guides/performance.md) | Limits, parallelism, scaling |
| [production-integration.md](guides/production-integration.md) | Embed in services, untrusted input |
| [protege-axiom-counts.md](guides/protege-axiom-counts.md) | Why counts differ from Protégé |
| [troubleshooting.md](guides/troubleshooting.md) | Common problems and fixes |
| [security.md](security.md) | Untrusted JSON and OWL input |
| [comparison.md](comparison.md) | OntoLogos vs ELK, Konclude, reasonable, whelk-rs |

## Reference

| Document | Description |
|----------|-------------|
| [architecture.md](architecture.md) | Crate graph and data flow |
| [errors.md](reference/errors.md) | Error enums (core, parser, profile, engines) |
| [cli.md](reference/cli.md) | `ontologos` command-line tool |
| [supported-constructs.md](reference/supported-constructs.md) | Mapped vs skipped OWL constructs |
| [reasonable-limits.md](reference/reasonable-limits.md) | Upstream reasonable adapter gaps |
| [explain.md](reference/explain.md) | Proof graphs (Rust, CLI, Python) |
| [query.md](reference/query.md) | Taxonomy query API |
| [rl-rules.md](reference/rl-rules.md) | OWL RL rule catalog |
| [conformance.md](reference/conformance.md) | HermiT-ported test coverage |
| [json-snapshot-v2.md](json-snapshot-v2.md) | JSON snapshot schema |
| [docs.rs/ontologos-core](https://docs.rs/ontologos-core/1.0.0) | Rust API (core) |
| [docs.rs/ontologos-bridge](https://docs.rs/ontologos-bridge/1.0.0) | Rust API (adapters) |
| [docs.rs/ontologos-parser](https://docs.rs/ontologos-parser/1.0.0) | Rust API (parser) |
| [docs.rs/ontologos-profile](https://docs.rs/ontologos-profile/1.0.0) | Rust API (profile) |
| [docs.rs/ontologos-rdfs](https://docs.rs/ontologos-rdfs/1.0.0) | Rust API (RDFS engine) |
| [docs.rs/ontologos-rl](https://docs.rs/ontologos-rl/1.0.0) | Rust API (OWL RL engine) |
| [docs.rs/ontologos-el](https://docs.rs/ontologos-el/1.0.0) | Rust API (OWL EL classifier) |
| [docs.rs/ontologos-query](https://docs.rs/ontologos-query/1.0.0) | Rust API (taxonomy queries) |
| [docs.rs/ontologos-explain](https://docs.rs/ontologos-explain/1.0.0) | Rust API (explanations) |

## Migration

| Document | Description |
|----------|-------------|
| [Upgrade to latest](migration/index.md) | Jump to v1.0.0 from any version |
| [v0.1-to-v0.2.md](migration/v0.1-to-v0.2.md) | Upgrade guide |
| [v0.2-to-v0.3.md](migration/v0.2-to-v0.3.md) | RDFS engine and materialize CLI |
| [v0.3.0-to-v0.3.1.md](migration/v0.3.0-to-v0.3.1.md) | Patch: classify CLI report |
| [v0.3.x-to-v0.4.0.md](migration/v0.3.x-to-v0.4.0.md) | ABox, OWL RL, Python `profile="rl"` |
| [v0.4.x-to-v0.5.0.md](migration/v0.4.x-to-v0.5.0.md) | OWL EL, CLI profile routing, Python `auto`/`el` |
| [v0.5.x-to-v0.6.0.md](migration/v0.5.x-to-v0.6.0.md) | Explanations, bridge crate, in-house EL (v0.6.1) |
| [v0.6.x-to-v0.7.0.md](migration/v0.6.x-to-v0.7.0.md) | Semver alignment release (no API changes) |
| [v0.7.x-to-v0.8.0.md](migration/v0.7.x-to-v0.8.0.md) | Incremental reasoning |
| [v0.8.x-to-v1.0.0.md](migration/v0.8.x-to-v1.0.0.md) | Python ecosystem API |

## Project

| Document | Description |
|----------|-------------|
| [faq.md](project/faq.md) | Common questions |
| [release-status.md](project/release-status.md) | Version and channel truth |
| [release-notes.md](project/release-notes.md) | Version highlights |
| [roadmap-summary.md](project/roadmap-summary.md) | Release plan overview |
| [changelog.md](project/changelog.md) | Release history |
| [contributing.md](project/contributing.md) | Development workflow |
| [benchmarks.md](project/benchmarks.md) | Benchmark corpora (maintainers) |
| [security-policy.md](project/security-policy.md) | Security reporting |
| [code-of-conduct.md](project/code-of-conduct.md) | Community standards |

Maintainer research notes: `docs/internal/research/` (not published on Read the Docs).
