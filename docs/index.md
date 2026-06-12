# OntoLogos Documentation

Native Rust ontology reasoning: load OWL files, detect profiles, run RDFS materialization and OWL RL saturation.

!!! warning "Early development (v0.5)"
    OntoLogos maps a **subset** of OWL axioms into its core model — `axiom_count()` is mapper output, not Protégé's total. CLI **`classify --profile auto`** routes to EL or RL; use **`materialize`** for explicit RDFS. Full OWL DL classification is not yet available. See [Supported constructs](reference/supported-constructs.md).

## Persona quick links

| I want to… | Start here |
|------------|------------|
| Try it in 5 minutes (no clone) | [Crates.io quick start](getting-started/index.md#cratesio-only-no-clone) |
| Try it from a clone | [Getting started index](getting-started/index.md) |
| Evaluate vs ELK / reasonable | [Comparison](comparison.md) |
| Integrate in Rust | [Choosing an API](guides/choosing-an-api.md) |
| Use Python | [Python guide](guides/python.md) |
| Contribute | [Contributing](project/contributing.md) |

## Capability matrix (v0.5)

| Capability | Library | CLI | Python |
|------------|---------|-----|--------|
| Load OWL files | Yes | Yes | Yes |
| Profile detection | Yes | `profile` | No |
| RDFS materialization | Yes | `materialize` / `classify --profile rdfs` | `profile="rdfs"` |
| OWL RL saturation | Yes | `classify --profile rl` | `profile="rl"` |
| OWL EL taxonomy | Yes | `classify --profile el` | `profile="el"` |
| Taxonomy queries | Yes (`ontologos-query`) | JSON output | `taxonomy` property |
| Materialization reports | Yes | Yes (RDFS/RL) | Yes |
| Export saturated ontology | Yes (in-process) | No | No |

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
| [json-snapshot-v2.md](json-snapshot-v2.md) | JSON v2 format, limits, migration |

## Guides

| Document | Description |
|----------|-------------|
| [choosing-an-api.md](guides/choosing-an-api.md) | Which crate and entry point to use |
| [profile-detection.md](guides/profile-detection.md) | OWL profile detection and diagnostics |
| [python.md](guides/python.md) | Python bindings (`pip install ontologos`) |
| [glossary.md](guides/glossary.md) | OWL and OntoLogos terminology |
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
| [rl-rules.md](reference/rl-rules.md) | OWL RL rule catalog |
| [conformance.md](reference/conformance.md) | HermiT-ported test coverage |
| [json-snapshot-v2.md](json-snapshot-v2.md) | JSON snapshot schema |
| [docs.rs/ontologos-core](https://docs.rs/ontologos-core/0.4.0) | Rust API (core) |
| [docs.rs/ontologos-parser](https://docs.rs/ontologos-parser/0.4.0) | Rust API (parser) |
| [docs.rs/ontologos-profile](https://docs.rs/ontologos-profile/0.4.0) | Rust API (profile) |
| [docs.rs/ontologos-rdfs](https://docs.rs/ontologos-rdfs/0.4.0) | Rust API (RDFS engine) |
| [docs.rs/ontologos-rl](https://docs.rs/ontologos-rl/0.4.0) | Rust API (OWL RL engine) |

## Migration

| Document | Description |
|----------|-------------|
| [v0.1-to-v0.2.md](migration/v0.1-to-v0.2.md) | Upgrade guide |
| [v0.2-to-v0.3.md](migration/v0.2-to-v0.3.md) | RDFS engine and materialize CLI |
| [v0.3.0-to-v0.3.1.md](migration/v0.3.0-to-v0.3.1.md) | Patch: classify CLI report |
| [v0.3.x-to-v0.4.0.md](migration/v0.3.x-to-v0.4.0.md) | ABox, OWL RL, Python `profile="rl"` |

## Project

| Document | Description |
|----------|-------------|
| [faq.md](project/faq.md) | Common questions |
| [roadmap-summary.md](project/roadmap-summary.md) | Release plan overview |
| [changelog.md](project/changelog.md) | Release history |
| [contributing.md](project/contributing.md) | Development workflow |
| [benchmarks.md](project/benchmarks.md) | Benchmark corpora (maintainers) |
| [security-policy.md](project/security-policy.md) | Security reporting |
| [code-of-conduct.md](project/code-of-conduct.md) | Community standards |

Maintainer research notes: `docs/internal/research/` (not published on Read the Docs).
