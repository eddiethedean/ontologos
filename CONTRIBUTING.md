# Contributing to OntoLogos

Thank you for your interest in contributing. OntoLogos is in early development (v0.6); high-impact contributions include incremental reasoning, conformance, and documentation.

## Prerequisites

- Rust **1.88+** (see `rust-version` in the workspace [Cargo.toml](Cargo.toml))
- `cargo fmt` and `cargo clippy` (installed via `rustup component add rustfmt clippy`)

## Getting started

```bash
git clone https://github.com/eddiethedean/ontologos.git
cd ontologos
./benchmarks/scripts/download.sh   # required for Pizza corpus and full test suite
cargo build
cargo test --workspace
```

> **Benchmark download:** `family.owl` is vendored, but Pizza and other corpora require `./benchmarks/scripts/download.sh`. CI runs this automatically. Without it, some parser and integration tests fail with `missing benchmark corpus pizza`.

Run the quick-start examples:

```bash
cargo run -p ontologos-core --example pizza_builder
cargo run -p ontologos-parser --example load_and_profile
cargo run -p ontologos-rl --example rl_saturation
```

## Documentation

Published at **[ontologos.readthedocs.io](https://ontologos.readthedocs.io/)** (MkDocs Material, built via [Read the Docs](https://readthedocs.org/)).

```bash
pip install -r docs/requirements.txt
NO_MKDOCS_2_WARNING=1 mkdocs serve
```

See [docs/readthedocs.md](docs/readthedocs.md) for import instructions and local builds (also linked from [Contributing](project/contributing.md)).

## Checks before opening a PR

CI runs the following on every push to `main` (see [.github/workflows/ci.yml](.github/workflows/ci.yml)):

```bash
./benchmarks/scripts/download.sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p ontologos-conformance
cargo build -p ontologos-cli --release
pip install -r docs/requirements.txt && NO_MKDOCS_2_WARNING=1 mkdocs build --strict
```

Run all locally before submitting.

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
| `crates/ontologos-rdfs/` | RDFS engine (v0.3) |
| `crates/ontologos-rl/` | OWL RL engine (v0.4) |
| `crates/ontologos-el/` | OWL EL engine (v0.5) |
| `crates/ontologos-query/` | Taxonomy queries (v0.5) |
| `crates/ontologos-conformance/` | HermiT-ported tests — [tests/hermit/README.md](tests/hermit/README.md) |
| `docs/` | User and reference documentation |
| `docs/internal/research/` | Maintainer research notes |
| `benchmarks/` | Benchmark ontology manifest and corpora |

See [Roadmap summary](docs/project/roadmap-summary.md) (full checklist: [ROADMAP.md on GitHub](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md)).

## Releases

### Release checklist

Before tagging a release (e.g. `v0.6.1`):

```bash
./benchmarks/scripts/download.sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
./benchmarks/scripts/compare-pizza-el-golden.sh
cargo test -p ontologos-conformance --locked
./benchmarks/scripts/compare-reasonable.sh
cargo publish -p ontologos-core --dry-run
cargo publish -p ontologos-profile --dry-run
cargo publish -p ontologos-parser --dry-run
cargo publish -p ontologos-bridge --dry-run
cargo publish -p ontologos-rdfs --dry-run
cargo publish -p ontologos-rl --dry-run --allow-dirty
cargo publish -p ontologos-el --dry-run --allow-dirty
cargo publish -p ontologos-query --dry-run --allow-dirty
cargo publish -p ontologos-explain --dry-run --allow-dirty
```

Use `--allow-dirty` only for local dry-runs before commit; release CI publishes from a clean tagged checkout.

Then:

1. Bump `version` in [crates/ontologos-py/pyproject.toml](crates/ontologos-py/pyproject.toml) and [python/ontologos/__init__.py](crates/ontologos-py/python/ontologos/__init__.py) to match the workspace version.
2. Ensure [CHANGELOG.md](CHANGELOG.md) has a dated version section and empty `[Unreleased]`.
3. Commit release prep on `main`.
4. Create an annotated tag: `git tag -a v0.6.1 -m "OntoLogos v0.6.1"`
5. Push commit and tag: `git push origin main && git push origin v0.6.1`
6. The [release workflow](.github/workflows/release.yml) runs when the tag is pushed (requires GitHub secrets below).
7. Create a GitHub Release from [`.github/release/v0.6.1.md`](.github/release/v0.6.1.md) (or the matching version file).

### Release secrets

| Secret | Purpose |
|--------|---------|
| `CARGO_REGISTRY_TOKEN` | Publish Rust crates to [crates.io](https://crates.io) |
| `PYPI_API_TOKEN` | Publish the `ontologos` Python package to [PyPI](https://pypi.org/project/ontologos/) |

Create a PyPI API token at https://pypi.org/manage/account/token/ (scope: entire account or project `ontologos`). Add it in the repo under **Settings → Secrets and variables → Actions**.

On each release tag, CI publishes:

- **crates.io** — crates listed in [.github/scripts/publish-crates.sh](.github/scripts/publish-crates.sh) (`ontologos-core`, `ontologos-bridge`, `ontologos-profile`, `ontologos-parser`, `ontologos-rdfs`, `ontologos-rl`, `ontologos-el`, `ontologos-query`, `ontologos-explain`, in dependency order)
- **PyPI** — `ontologos` via release CI (`maturin-action`): Linux (x86_64, aarch64), macOS (x86_64, aarch64), Windows (x64, aarch64), plus sdist. Manual upload: [.github/scripts/publish-pypi.sh](.github/scripts/publish-pypi.sh)

- **Tags:** Release tags follow semver (`v0.6.1`, …)
- **CHANGELOG:** [Keep a Changelog](https://keepachangelog.com/) format in [CHANGELOG.md](CHANGELOG.md)

## Questions

Open a [GitHub issue](https://github.com/eddiethedean/ontologos/issues) for bugs, feature requests, or design questions. Check [FAQ.md](FAQ.md) and [Troubleshooting](docs/guides/troubleshooting.md) first.

There is no Discord or mailing list — GitHub Issues is the primary support channel.
