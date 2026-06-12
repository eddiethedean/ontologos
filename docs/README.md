# Documentation Index

Welcome to OntoLogos documentation. Start here if you are new to the project.

## Learning path

1. **[README](../README.md)** — install, what works in v0.3, quick start
2. **[First ontology](getting-started/first-ontology.md)** — builder API walkthrough
3. **[Load an OWL file](getting-started/load-owl-file.md)** — parser, formats, `ParseMeta`
4. **[Profile detection](guides/profile-detection.md)** — EL/RL/QL/DL + hybrid diagnostics
5. **[JSON snapshots](json-snapshot-v2.md)** — load and save ontologies
6. **[Error reference](reference/errors.md)** — core, parser, profile errors
7. **[ROADMAP](../ROADMAP.md)** — what ships next

## Getting started

| Document | Description |
|----------|-------------|
| [first-ontology.md](getting-started/first-ontology.md) | Build a taxonomy with `OntologyBuilder` |
| [load-owl-file.md](getting-started/load-owl-file.md) | Load OWL/RDF files with `ontologos-parser` |
| [json-snapshot-v2.md](json-snapshot-v2.md) | JSON v2 format, limits, migration |

## Guides

| Document | Description |
|----------|-------------|
| [profile-detection.md](guides/profile-detection.md) | OWL profile detection and diagnostics |
| [troubleshooting.md](guides/troubleshooting.md) | Common problems and fixes |
| [security.md](security.md) | Untrusted JSON and OWL input |
| [comparison.md](comparison.md) | OntoLogos vs ELK, Konclude, reasonable, whelk-rs |
| [../benchmarks/README.md](../benchmarks/README.md) | Benchmark corpora and testing |
| [../FAQ.md](../FAQ.md) | Common questions |

## Reference

| Document | Description |
|----------|-------------|
| [errors.md](reference/errors.md) | Error enums (core, parser, profile) |
| [cli.md](reference/cli.md) | `ontologos` command-line tool |
| [supported-constructs.md](reference/supported-constructs.md) | Mapped vs skipped OWL constructs |
| [json-snapshot-v2.md](json-snapshot-v2.md) | JSON snapshot schema |
| [docs.rs/ontologos-core](https://docs.rs/ontologos-core) | Rust API (core) |
| [docs.rs/ontologos-parser](https://docs.rs/ontologos-parser) | Rust API (parser) |
| [docs.rs/ontologos-profile](https://docs.rs/ontologos-profile) | Rust API (profile) |
| [docs.rs/ontologos-rdfs](https://docs.rs/ontologos-rdfs) | Rust API (RDFS engine) |

## Migration

| Document | Description |
|----------|-------------|
| [v0.1-to-v0.2.md](migration/v0.1-to-v0.2.md) | Upgrade guide |
| [v0.2-to-v0.3.md](migration/v0.2-to-v0.3.md) | RDFS engine and materialize CLI |

## Project meta

| Document | Description |
|----------|-------------|
| [../ROADMAP.md](../ROADMAP.md) | Canonical release plan |
| [../CHANGELOG.md](../CHANGELOG.md) | Release history |
| [../CONTRIBUTING.md](../CONTRIBUTING.md) | Development workflow |
| [../SPEC.md](../SPEC.md) | Technical specification (status-tagged) |

Maintainer research notes: [internal/research/](internal/research/) (optional reading).
