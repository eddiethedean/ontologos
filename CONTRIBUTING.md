# Contributing to OntoLogos

Thank you for your interest in contributing. OntoLogos is in early development (v0.2); high-impact contributions include parser mapping, profile detection, tests, and documentation.

## Prerequisites

- Rust **1.78+** (see `rust-version` in the workspace [Cargo.toml](Cargo.toml))
- `cargo fmt` and `cargo clippy` (installed via `rustup component add rustfmt clippy`)

## Getting started

```bash
git clone https://github.com/eddiethedean/ontologos.git
cd ontologos
./benchmarks/scripts/download.sh   # required for parser/CLI integration tests
cargo build
cargo test --workspace
```

Run the quick-start example:

```bash
cargo run -p ontologos-core --example pizza_builder
cargo run -p ontologos-parser --example load_and_profile
```

## Checks before opening a PR

CI runs the following on every push to `main` (see [.github/workflows/ci.yml](.github/workflows/ci.yml)):

```bash
./benchmarks/scripts/download.sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build -p ontologos-cli --release
```

Run all five locally before submitting.

## Pull request guidelines

1. **Scope:** One logical change per PR when possible.
2. **Tests:** Add or update tests for behavior changes (core, parser, profile, CLI as appropriate).
3. **Docs:** Update README, CHANGELOG, or `docs/` when user-visible behavior changes.
4. **Breaking changes:** Note them in CHANGELOG under `[Unreleased]` or the target version.
5. **No `unsafe`:** The workspace forbids unsafe code.

## Project structure

| Path | Purpose |
|------|---------|
| `crates/ontologos-core/` | Data model |
| `crates/ontologos-parser/` | OWL/RDF file loading (v0.2) |
| `crates/ontologos-profile/` | Profile detection (v0.2) |
| `docs/` | User and reference documentation |
| `docs/internal/research/` | Maintainer research notes |
| `benchmarks/` | Benchmark ontology manifest and corpora |

See [ROADMAP.md](ROADMAP.md) for milestone ownership.

## Releases

### Release checklist

Before tagging a release (e.g. `v0.2.0`):

```bash
./benchmarks/scripts/download.sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
cargo publish -p ontologos-core --dry-run
cargo publish -p ontologos-parser --dry-run
cargo publish -p ontologos-profile --dry-run
```

Then:

1. Ensure [CHANGELOG.md](CHANGELOG.md) has a dated version section and empty `[Unreleased]`.
2. Commit release prep on `main`.
3. Create an annotated tag: `git tag -a v0.2.0 -m "OntoLogos v0.2.0"`
4. Push commit and tag: `git push origin main && git push origin v0.2.0`
5. The [release workflow](.github/workflows/release.yml) runs when the tag is pushed (requires GitHub secrets below).
6. Create a GitHub Release from [`.github/release/v0.2.0.md`](.github/release/v0.2.0.md) (or the matching version file).

### Release secrets

| Secret | Purpose |
|--------|---------|
| `CARGO_REGISTRY_TOKEN` | Publish Rust crates to [crates.io](https://crates.io) |
| `PYPI_API_TOKEN` | Publish the `ontologos` Python package to [PyPI](https://pypi.org/project/ontologos/) |

Create a PyPI API token at https://pypi.org/manage/account/token/ (scope: entire account or project `ontologos`). Add it in the repo under **Settings → Secrets and variables → Actions**.

On each release tag, CI publishes:

- **crates.io** — crates listed in [.github/scripts/publish-crates.sh](.github/scripts/publish-crates.sh) (v0.2: `ontologos-core`, `ontologos-parser`, `ontologos-profile`)
- **PyPI** — `ontologos` via [.github/scripts/publish-pypi.sh](.github/scripts/publish-pypi.sh) (`maturin`, Linux wheel + sdist)

Bump `version` in [crates/ontologos-py/pyproject.toml](crates/ontologos-py/pyproject.toml) and [python/ontologos/__init__.py](crates/ontologos-py/python/ontologos/__init__.py) to match the workspace version before tagging.

Manual PyPI publish (optional):

```bash
PYPI_API_TOKEN=pypi-... ./.github/scripts/publish-pypi.sh
```

- **Tags:** Release tags follow semver (`v0.2.0`, …)
- **CHANGELOG:** [Keep a Changelog](https://keepachangelog.com/) format in [CHANGELOG.md](CHANGELOG.md)

## Questions

Open a GitHub issue for bugs, feature requests, or design questions. Check [FAQ.md](FAQ.md) first.
