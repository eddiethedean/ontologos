# Contributing to OntoLogos

Thank you for your interest in contributing. OntoLogos is in early development (v0.1); the highest-impact contributions right now are core data model work, tests, and documentation.

## Prerequisites

- Rust **1.78+** (see `rust-version` in the workspace [Cargo.toml](Cargo.toml))
- `cargo fmt` and `cargo clippy` (installed via `rustup component add rustfmt clippy`)

## Getting started

```bash
git clone https://github.com/eddiethedean/ontologos.git
cd ontologos
cargo build
cargo test --workspace
```

Run the quick-start example:

```bash
cargo run -p ontologos-core --example pizza_builder
```

## Checks before opening a PR

CI runs the following on every push to `main` (see [.github/workflows/ci.yml](.github/workflows/ci.yml)):

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build -p ontologos-cli --release
```

Run all four locally before submitting.

## Pull request guidelines

1. **Scope:** One logical change per PR when possible.
2. **Tests:** Add or update tests for behavior changes in `ontologos-core`.
3. **Docs:** Update README, CHANGELOG, or `docs/` when user-visible behavior changes.
4. **Breaking changes:** Note them in CHANGELOG under `[Unreleased]` or the target version.
5. **No `unsafe`:** The workspace forbids unsafe code.

## Project structure

| Path | Purpose |
|------|---------|
| `crates/ontologos-core/` | Data model (v0.1 focus) |
| `crates/ontologos-parser/` | File loading (v0.2) |
| `docs/` | User and reference documentation |
| `docs/internal/research/` | Maintainer research notes |
| `benchmarks/` | Benchmark ontology manifest |

See [ROADMAP.md](ROADMAP.md) for milestone ownership.

## Releases

- **crates.io:** v0.1 publishes `ontologos-core` only via [.github/scripts/publish-crates.sh](.github/scripts/publish-crates.sh)
- **Tags:** Release tags follow semver (`v0.1.0`, …)
- **CHANGELOG:** [Keep a Changelog](https://keepachangelog.com/) format in [CHANGELOG.md](CHANGELOG.md)

Maintainers: tag after merging release prep, then run the release workflow or publish script.

## Questions

Open a GitHub issue for bugs, feature requests, or design questions. Check [FAQ.md](FAQ.md) first.
