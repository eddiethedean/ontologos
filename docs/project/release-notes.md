# Release notes

Annotated release summaries live in the repository under [`.github/release/`](https://github.com/eddiethedean/ontologos/tree/main/.github/release).

| Version | Theme | Notes |
|---------|-------|-------|
| [v1.1.3](https://github.com/eddiethedean/ontologos/blob/main/.github/release/v1.1.3.md) | Test verification patch | Conformance guard honesty; stronger oracle-backed tests |
| [v1.1.2](https://github.com/eddiethedean/ontologos/blob/main/.github/release/v1.1.2.md) | Bug-fix patch | Parser, DL, SWRL, RL, bindings; docs migration links |
| [v1.1.0](https://github.com/eddiethedean/ontologos/blob/main/.github/release/v1.1.0.md) | Multi-language bindings | Java, .NET, C/C++, shared FFI; Node/WASM over `ontologos-js` |
| [v1.0.0](https://github.com/eddiethedean/ontologos/blob/main/.github/release/v1.0.0.md) | HermiT parity milestone | DL + SWRL stable on crates.io/PyPI; 1048 active conformance tests |
| [v0.9.0](https://github.com/eddiethedean/ontologos/blob/main/.github/release/v0.9.0.md) | Python ecosystem | `Ontology`, `explain()`, incremental Python, DataFrame export |
| [v0.8.0](https://github.com/eddiethedean/ontologos/blob/main/.github/release/v0.8.0.md) | Incremental reasoning | Dirty tracking, sessions, `--incremental` |
| [v0.7.0](https://github.com/eddiethedean/ontologos/blob/main/.github/release/v0.7.0.md) | Dependency-first adapters | Bridge crate, semver alignment |
| [v0.6.0](https://github.com/eddiethedean/ontologos/blob/main/.github/release/v0.6.0.md) | Explanations | Proof graphs, CLI `explain` |
| [v0.5.0](https://github.com/eddiethedean/ontologos/blob/main/.github/release/v0.5.0.md) | OWL EL & query | EL classification, CLI profile routing |

Full history: [CHANGELOG](changelog.md) on GitHub.

## v1.1.3 highlights

- **Conformance:** remove weak Consistent-but-all-unsat IRI-shape entailment shortcut; defer case until DL proves named-class ⊥
- **Tests:** oracle-backed facade/contract/profile/EL/Python asserts; shared semantic fixtures
- **No breaking Rust/Python API changes** — bump pins to `"1.1.3"`

## v1.1.0 highlights

- **`ontologos-ffi`:** shared C ABI for Java, .NET, and C/C++ hosts
- **Java / .NET / C/C++:** native bindings with smoke tests in CI (`scripts/ci-bindings.sh`)
- **Node / WASM:** N-API and wasm-pack bindings over `ontologos-js`
- **No breaking Rust/Python API changes** — bump pins to `"1.1.0"`

See [v1.0.x → v1.1.0 migration](../migration/v1.0.x-to-v1.1.0.md) and [Install channels](../guides/install-channels.md).

## v1.0.0 highlights

See [Profile stability matrix](../guides/profile-stability.md) for canonical status. Highlights:

- **`ontologos-facade`:** unified `classify()` for CLI, Python, and multi-profile Rust apps
- **`Profile::Auto` + DL:** DL-detected ontologies route through `ontologos-dl` hybrid classifier
- **`dl` and `swrl`:** stable on **PyPI / crates.io 1.0.0**
- **`alc`, `dl-preview`:** preview profiles — see [Preview profiles](../guides/preview-profiles.md)

See [Facade API](../guides/facade-api.md) and [Architecture](../architecture.md).

## Latest upgrade

Most users upgrading today: pin **1.1.3** or see [v1.0.x → v1.1.0 migration](../migration/v1.0.x-to-v1.1.0.md).
