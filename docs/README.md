# Documentation Index

Welcome to OntoLogos documentation. Start here if you are new to the project.

## Persona quick links

| I want to… | Start here |
|------------|------------|
| Try it in 5 minutes | [Getting started index](getting-started/index.md) |
| Evaluate vs ELK / reasonable | [Comparison](comparison.md) |
| Integrate in Rust | [Choosing an API](guides/choosing-an-api.md) |
| Use Python | [Python guide](guides/python.md) |
| Contribute | [CONTRIBUTING](../CONTRIBUTING.md) |

## Learning path

1. **[README](../README.md)** — install, what works in v0.4, quick start
2. **[First ontology](getting-started/first-ontology.md)** — builder API walkthrough
3. **[Load an OWL file](getting-started/load-owl-file.md)** — parser, formats, `ParseMeta`
4. **[OWL RL saturation](getting-started/owl-rl-saturation.md)** — `RlEngine::saturate`, reports, clashes
5. **[Profile detection](guides/profile-detection.md)** — EL/RL/QL/DL + hybrid diagnostics
6. **[JSON snapshots](json-snapshot-v2.md)** — load and save ontologies
7. **[Error reference](reference/errors.md)** — core, parser, profile, engine errors
8. **[ROADMAP](../ROADMAP.md)** — what ships next

## Getting started

| Document | Description |
|----------|-------------|
| [index.md](getting-started/index.md) | 5-minute success paths by persona |
| [first-ontology.md](getting-started/first-ontology.md) | Build a taxonomy with `OntologyBuilder` |
| [load-owl-file.md](getting-started/load-owl-file.md) | Load OWL/RDF files with `ontologos-parser` |
| [owl-rl-saturation.md](getting-started/owl-rl-saturation.md) | OWL RL forward-chaining with `ontologos-rl` |
| [json-snapshot-v2.md](json-snapshot-v2.md) | JSON v2 format, limits, migration |

## Guides

| Document | Description |
|----------|-------------|
| [choosing-an-api.md](guides/choosing-an-api.md) | Which crate and entry point to use |
| [profile-detection.md](guides/profile-detection.md) | OWL profile detection and diagnostics |
| [python.md](guides/python.md) | Python bindings (`pip install ontologos`) |
| [troubleshooting.md](guides/troubleshooting.md) | Common problems and fixes |
| [security.md](security.md) | Untrusted JSON and OWL input |
| [comparison.md](comparison.md) | OntoLogos vs ELK, Konclude, reasonable, whelk-rs |
| [../benchmarks/README.md](../benchmarks/README.md) | Benchmark corpora and testing |
| [../FAQ.md](../FAQ.md) | Common questions |

## Reference

| Document | Description |
|----------|-------------|
| [architecture.md](architecture.md) | Crate graph and data flow |
| [errors.md](reference/errors.md) | Error enums (core, parser, profile, engines) |
| [cli.md](reference/cli.md) | `ontologos` command-line tool |
| [supported-constructs.md](reference/supported-constructs.md) | Mapped vs skipped OWL constructs |
| [rl-rules.md](reference/rl-rules.md) | OWL RL rule catalog |
| [conformance.md](reference/conformance.md) | HermiT-ported test coverage for evaluators |
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
| [v0.3.0-to-v0.3.1.md](migration/v0.3.0-to-v0.3.1.md) | Patch: classify CLI report, docs, delegate hint |
| [v0.3.x-to-v0.4.0.md](migration/v0.3.x-to-v0.4.0.md) | ABox, OWL RL engine, Python `profile="rl"` |

## Project meta

| Document | Description |
|----------|-------------|
| [../ROADMAP.md](../ROADMAP.md) | Canonical release plan |
| [../CHANGELOG.md](../CHANGELOG.md) | Release history |
| [../CONTRIBUTING.md](../CONTRIBUTING.md) | Development workflow |
| [../SPEC.md](../SPEC.md) | Technical specification (status-tagged) |
| [../SECURITY.md](../SECURITY.md) | Security reporting |
| [../CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md) | Community standards |

Maintainer research notes: [internal/research/](internal/research/) (optional reading).
