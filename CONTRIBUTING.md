# Contributing to OntoLogos

Thank you for your interest in contributing. OntoLogos is in active development: **v0.9.0** is the latest published release on crates.io/PyPI; the **`main`** branch tracks workspace **1.0.0** (pre-release). High-impact contributions include conformance, Python bindings, documentation, and incremental reasoning polish.

## Prerequisites

- Rust **1.88+** (see `rust-version` in the workspace [Cargo.toml](Cargo.toml))
- `cargo fmt` and `cargo clippy` (installed via `rustup component add rustfmt clippy`)
- For Python bindings: Python **3.10+**, `maturin`, `pytest`

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

## Python bindings development

```bash
cd crates/ontologos-py
python -m venv .venv
source .venv/bin/activate   # Windows: .venv\Scripts\activate
pip install 'maturin>=1.7,<2.0' pytest '.[pandas]'
maturin develop --release
pytest tests/ -q
```

From the repo root (after `./benchmarks/scripts/download.sh`):

```bash
cd crates/ontologos-py && source .venv/bin/activate && pytest tests/test_pizza_golden.py -q
```

See [crates/ontologos-py/README.md](crates/ontologos-py/README.md) for the full Python API.

## Documentation

Published at **[ontologos.readthedocs.io](https://ontologos.readthedocs.io/)** (MkDocs Material, built via [Read the Docs](https://readthedocs.org/)).

```bash
pip install -r docs/requirements.txt
NO_MKDOCS_2_WARNING=1 mkdocs serve
```

See [docs/readthedocs.md](docs/readthedocs.md) for import instructions and local builds (also linked from [Contributing](docs/project/contributing.md)).

When changing user-facing docs, run the version consistency check:

```bash
chmod +x docs/scripts/check-doc-versions.sh
./docs/scripts/check-doc-versions.sh
```

## Checks before opening a PR

CI runs the following on every push to `main` (see [.github/workflows/ci.yml](.github/workflows/ci.yml)). Run all locally before submitting:

```bash
./benchmarks/scripts/download.sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
./benchmarks/scripts/compare-pizza-el-golden.sh
cargo test -p ontologos-conformance --locked
./benchmarks/scripts/compare-reasonable.sh
cargo test -p ontologos-el --test incremental_correctness --locked
cargo build -p ontologos-cli --release
chmod +x docs/scripts/check-doc-versions.sh
./docs/scripts/check-doc-versions.sh
chmod +x docs/build-site.sh
./docs/build-site.sh
```

Python (Linux CI parity):

```bash
cd crates/ontologos-py
python -m venv .venv && source .venv/bin/activate
pip install 'maturin>=1.7,<2.0' pytest '.[pandas]'
maturin develop --release
pytest tests/ -q
```

## Pull request guidelines

1. **Scope:** One logical change per PR when possible.
2. **Tests:** Add or update tests for behavior changes (core, parser, profile, CLI, Python as appropriate).
3. **Docs:** Update README, CHANGELOG, or `docs/` when user-visible behavior changes. Bump version pins to match [Cargo.toml](Cargo.toml).
4. **Breaking changes:** Note them in CHANGELOG under `[Unreleased]` or the target version.
5. **No `unsafe`:** The workspace forbids unsafe code.

## Project structure

| Path | Purpose |
|------|---------|
| `crates/ontologos-core/` | Data model, builder, JSON v2 |
| `crates/ontologos-parser/` | OWL/RDF file loading |
| `crates/ontologos-profile/` | Profile detection |
| `crates/ontologos-bridge/` | core ↔ horned-owl/reasonable adapters |
| `crates/ontologos-rdfs/` | RDFS facade → reasonable |
| `crates/ontologos-rl/` | OWL RL facade → reasonable |
| `crates/ontologos-el/` | OWL EL completion engine |
| `crates/ontologos-query/` | Taxonomy queries |
| `crates/ontologos-explain/` | Proof graphs and explanations |
| `crates/ontologos-cli/` | CLI binary (not published) |
| `crates/ontologos-py/` | Python bindings (PyPI) |
| `crates/ontologos-watch/` | File-watch reload hook (workspace only) |
| `crates/ontologos-conformance/` | HermiT-ported tests |
| `docs/` | User and reference documentation |
| `docs/internal/research/` | Maintainer research notes |
| `benchmarks/` | Benchmark ontology manifest and corpora |

See [Roadmap summary](docs/project/roadmap-summary.md) (full checklist: [ROADMAP.md on GitHub](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md)).

## Releases

### Release checklist

Before tagging a release (e.g. `v0.9.0`):

```bash
./benchmarks/scripts/download.sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
./benchmarks/scripts/compare-pizza-el-golden.sh
cargo test -p ontologos-conformance --locked
./benchmarks/scripts/compare-reasonable.sh
./docs/scripts/check-doc-versions.sh
cargo publish -p ontologos-core --dry-run
```

Create or update [`.github/release/vX.Y.Z.md`](.github/release/) with highlights, version bumps, migration guide link, and pre-release checklist.

`cargo publish --dry-run` for downstream crates requires prior crates at the new version on crates.io (or use `cargo package -p ontologos-core --allow-dirty` per crate in publish order). On release, CI publishes in dependency order via [.github/scripts/publish-crates.sh](.github/scripts/publish-crates.sh).

Optional full local packaging check:

```bash
for crate in ontologos-core ontologos-profile ontologos-parser ontologos-bridge ontologos-rdfs ontologos-rl ontologos-el ontologos-query ontologos-explain; do
  cargo package -p "$crate" --allow-dirty
done
```

Then:

1. Bump `version` in workspace [Cargo.toml](Cargo.toml), [crates/ontologos-py/pyproject.toml](crates/ontologos-py/pyproject.toml), and [python/ontologos/__init__.py](crates/ontologos-py/python/ontologos/__init__.py).
2. Ensure [CHANGELOG.md](CHANGELOG.md) has a dated version section and empty `[Unreleased]`.
3. Update version pins in `docs/getting-started/`, [FAQ.md](FAQ.md), and run `./docs/scripts/check-doc-versions.sh`.
4. Commit release prep on `main`.
5. Create an annotated tag: `git tag -a v0.9.0 -m "OntoLogos v0.9.0"`
6. Push commit and tag: `git push origin main && git push origin v0.9.0`
7. The [release workflow](.github/workflows/release.yml) runs when the tag is pushed (requires GitHub secrets below).
8. Create a GitHub Release from [`.github/release/v0.9.0.md`](.github/release/v0.9.0.md) (or the matching version file).

### Release secrets

| Secret | Purpose |
|--------|---------|
| `CARGO_REGISTRY_TOKEN` | Publish Rust crates to [crates.io](https://crates.io) |
| `PYPI_API_TOKEN` | Publish the `ontologos` Python package to [PyPI](https://pypi.org/project/ontologos/) |

Create a PyPI API token at https://pypi.org/manage/account/token/ (scope: entire account or project `ontologos`). Add it in the repo under **Settings → Secrets and variables → Actions**.

On each release tag, CI publishes:

- **crates.io** — crates listed in [.github/scripts/publish-crates.sh](.github/scripts/publish-crates.sh) (`ontologos-core`, `ontologos-bridge`, `ontologos-profile`, `ontologos-parser`, `ontologos-rdfs`, `ontologos-rl`, `ontologos-el`, `ontologos-query`, `ontologos-explain`, in dependency order)
- **PyPI** — `ontologos` via release CI (`maturin-action`): Linux (x86_64, aarch64), macOS (x86_64, aarch64), Windows (x64, aarch64), plus sdist. Manual upload: [.github/scripts/publish-pypi.sh](.github/scripts/publish-pypi.sh)

- **Tags:** Release tags follow semver (`v0.9.0`, …)
- **CHANGELOG:** [Keep a Changelog](https://keepachangelog.com/) format in [CHANGELOG.md](CHANGELOG.md)

## Questions

Open a [GitHub issue](https://github.com/eddiethedean/ontologos/issues) for bugs, feature requests, or design questions. Check [FAQ.md](FAQ.md) and [Troubleshooting](docs/guides/troubleshooting.md) first.

There is no Discord or mailing list — GitHub Issues is the primary support channel.

See also [Code of Conduct](CODE_OF_CONDUCT.md) and [Security policy](SECURITY.md).
