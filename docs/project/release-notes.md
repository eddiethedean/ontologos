# Release notes

Annotated release summaries live in the repository under [`.github/release/`](https://github.com/eddiethedean/ontologos/tree/main/.github/release).

| Version | Theme | Notes |
|---------|-------|-------|
| [v0.9.0](https://github.com/eddiethedean/ontologos/blob/main/.github/release/v0.9.0.md) | Python ecosystem | `Ontology`, `explain()`, incremental Python, DataFrame export |
| [v0.8.0](https://github.com/eddiethedean/ontologos/blob/main/.github/release/v0.8.0.md) | Incremental reasoning | Dirty tracking, sessions, `--incremental` |
| [v0.7.0](https://github.com/eddiethedean/ontologos/blob/main/.github/release/v0.7.0.md) | Dependency-first adapters | Bridge crate, semver alignment |
| [v0.6.0](https://github.com/eddiethedean/ontologos/blob/main/.github/release/v0.6.0.md) | Explanations | Proof graphs, CLI `explain` |
| [v0.5.0](https://github.com/eddiethedean/ontologos/blob/main/.github/release/v0.5.0.md) | OWL EL & query | EL classification, CLI profile routing |

Full history: [CHANGELOG](changelog.md) on GitHub.

## Since v0.9.0 on `main` (facade / DL preview)

Documented in [Preview profiles](../guides/preview-profiles.md). Highlights:

- **`ontologos-facade`:** unified `classify()` for CLI, Python, and multi-profile Rust apps
- **`Profile::Auto` + DL:** DL-detected ontologies route through `ontologos-dl` hybrid classifier
- **CLI/Python profiles:** `alc`, `dl`, `dl-preview`, `swrl` (preview; SWRL not executable)
- **DL correctness fixes:** domain clause wildcards, role subsumption direction, `ResourceLimit` on budget exhaustion, taxonomy merge preserves equivalences
- **Routing fixes:** EL path no longer misroutes `Profile::Dl`; SWRL returns explicit errors instead of silent DL classify

See [Facade API](../guides/facade-api.md) and [Architecture](../architecture.md).

## Latest upgrade

Most users upgrading today: [v0.8.x → v0.9.0 migration](../migration/v0.8.x-to-v0.9.0.md).
