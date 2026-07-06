# Contributing to OntoLogos

Thank you for your interest in contributing. OntoLogos is in active development: **v1.1.0** is the current release on crates.io/PyPI and the **`main`** workspace. See [Release status](docs/project/release-status.md). High-impact contributions include conformance, bindings, documentation, and incremental reasoning polish.

## Prerequisites

- Rust **1.88+** (MSRV — see `rust-version` in the workspace [Cargo.toml](Cargo.toml); CI enforces 1.88)
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
cargo run -p ontologos-facade --example facade_auto -- benchmarks/data/family.owl
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
./docs/serve-site.sh   # runs version + snippet checks, then mkdocs serve
```

Or `NO_MKDOCS_2_WARNING=1 mkdocs serve` after running the check scripts manually.

See [docs/readthedocs.md](docs/readthedocs.md) for import instructions and local builds (also linked from [Contributing](docs/project/contributing.md)).

**FAQ and CONTRIBUTING:** Edit the root [`FAQ.md`](FAQ.md) and [`CONTRIBUTING.md`](CONTRIBUTING.md) only — the docs site includes them via `include-markdown`. Do not duplicate content in `docs/project/faq.md`.

When changing user-facing docs, run the version and snippet consistency checks:

```bash
chmod +x docs/scripts/check-doc-versions.sh docs/scripts/check-doc-snippets.sh
./docs/scripts/check-doc-versions.sh
./docs/scripts/check-doc-snippets.sh
```

## Which tests to run

| Change type | Usually enough |
|-------------|----------------|
| `docs/` only | `./docs/build-site.sh` (includes version + snippet checks) |
| Facade / CLI / Python public API | `cargo test -p ontologos-contract --release` |
| Single engine crate | `cargo test -p ontologos-el` (or affected crate) |
| HermiT catalog / DL internals | `cargo test -p ontologos-conformance --release` (~26 min; nightly/release CI) |
| Full CI parity | See [Full CI parity](#full-ci-parity-engine-conformance-or-workspace-wide-changes) below |

`ontologos-contract` is the **Tier 0 public API contract** — what CLI and Python depend on. `ontologos-conformance` is the HermiT parity harness for engine contributors.

## Checks before opening a PR

### Light path (docs, examples, or single-crate fixes)

Usually sufficient:

```bash
cargo fmt --all -- --check
cargo clippy -p <affected-crate> --all-targets -- -D warnings
cargo test -p <affected-crate>
./docs/scripts/check-doc-versions.sh   # when docs/ or README version pins change
./docs/scripts/check-doc-snippets.sh   # when docs/ examples or API references change
```

### Full CI parity (engine, conformance, or workspace-wide changes)

CI on every push/PR to `main` (see [.github/workflows/ci.yml](.github/workflows/ci.yml)):

```bash
./benchmarks/scripts/download.sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --exclude ontologos-conformance --exclude ontologos-contract --locked
cargo test -p ontologos-contract --release --locked
./benchmarks/scripts/check-hermit-ignore-budget.sh
./benchmarks/scripts/compare-pizza-el-golden.sh
./benchmarks/scripts/compare-classification-fixtures.sh
./benchmarks/scripts/check-pr-gates.sh
./benchmarks/scripts/check-hermit-parity-phases.sh
./benchmarks/scripts/check-hermit-catalog.sh
./benchmarks/scripts/compare-tier-c-gate.sh
./benchmarks/scripts/compare-hermit-tier-c.sh
./benchmarks/scripts/compare-reasonable.sh
cargo test -p ontologos-el --test incremental_correctness --locked
cargo build -p ontologos-cli --release
# Tier C strict family gate requires Java 17 + HermiT JAR (CI only):
# ./benchmarks/scripts/download-hermit-jar.sh && ./benchmarks/scripts/compare-tier-c-strict-family.sh
chmod +x docs/scripts/check-doc-versions.sh docs/scripts/check-doc-snippets.sh
./docs/scripts/check-doc-versions.sh
./docs/scripts/check-doc-snippets.sh
chmod +x docs/build-site.sh
./docs/build-site.sh
```

**Not on every PR:** `cargo test -p ontologos-conformance` runs on [conformance nightly](.github/workflows/conformance-nightly.yml) and release workflows (~26 min). Run locally when changing DL engine internals or HermiT catalog cases.

Python (Linux CI parity):

```bash
cd crates/ontologos-py
python -m venv .venv && source .venv/bin/activate
pip install 'maturin>=1.7,<2.0' pytest '.[pandas,polars]'
maturin develop --release
pytest tests/ -q
```

## Pull request guidelines

1. **Scope:** One logical change per PR when possible.
2. **Tests:** Add or update tests for behavior changes (core, parser, profile, CLI, Python as appropriate).
3. **Docs:** Update README, CHANGELOG, or `docs/` when user-visible behavior changes.
4. **Version pins in docs:** Published install blocks must use **1.1.0** (`docs/scripts/check-doc-versions.sh` enforces this). Migration guides may reference older versions for upgrade paths.
5. **Breaking changes:** Note them in CHANGELOG under `[Unreleased]` or the target version.
6. **No `unsafe`:** The workspace forbids unsafe code.

## Project structure

| Path | Purpose |
|------|---------|
| `crates/ontologos-core/` | Data model, builder, JSON v3 (v2 read) |
| `crates/ontologos-parser/` | OWL/RDF file loading |
| `crates/ontologos-profile/` | Profile detection |
| `crates/ontologos-bridge/` | core ↔ horned-owl/reasonable adapters |
| `crates/ontologos-rl/` | OWL RL, RDFS, and ABox facades → reasonable |
| `crates/ontologos-el/` | OWL EL completion engine |
| `crates/ontologos-alc/` | ALC tableau-lite (preview) |
| `crates/ontologos-dl/` | OWL 2 DL reasoner |
| `crates/ontologos-swrl/` | DLSafe SWRL + DL |
| `crates/ontologos-ql/` | OWL QL queries |
| `crates/ontologos-facade/` | Unified classify routing |
| `crates/ontologos-explain/` | Proof graphs and explanations |
| `crates/ontologos-cli/` | CLI binary (not on crates.io) |
| `crates/ontologos-py/` | Python bindings (PyPI) |
| `crates/ontologos-contract/` | Public facade API contract tests (Tier 0) |
| `crates/ontologos-conformance/` | HermiT-ported tests |
| `docs/` | User and reference documentation |
| `docs/internal/` | Maintainer roadmap, ADRs, parity notes |
| `benchmarks/` | Benchmark ontology manifest and corpora |

See [Roadmap summary](docs/project/roadmap-summary.md) (full checklist: [internal roadmap](https://github.com/eddiethedean/ontologos/blob/main/docs/internal/roadmap.md)).

## Issue labels

Use GitHub issue templates when opening bugs or doc fixes. Recommended labels (create in repo settings if missing):

| Label | Use for |
|-------|---------|
| `good first issue` | Docs, examples, small isolated fixes |
| `documentation` | Read the Docs / README / migration guides |
| `bug` | Incorrect results, panics, parser failures |
| `enhancement` | New features or API proposals |

## Working on HermiT parity (v1.0 burndown)

If you are fixing DL engine gaps, porting HermiT tests, or burning down conformance failures, start with the **[HermiT burndown guide](docs/guides/hermit-burndown.md)**.

```bash
./benchmarks/scripts/download.sh
bash benchmarks/scripts/hermit-burndown.sh status   # parity %, backlog, next steps
bash benchmarks/scripts/hermit-burndown.sh loop     # daily fix-verify loop
```

**Remember:** PR CI runs contract tests, gate scripts, and HermiT parity phase checks @ 30s — not the full `ontologos-conformance` crate (see nightly workflow). Ian/ComplexConcept CE gaps are in `EXCLUDED_IDS` until tableau soundness closes. Run `hermit-burndown.sh promote` after fixing excluded cases.

Catalog mechanics: [tests/hermit/README.md](tests/hermit/README.md).

## Releases

**Next publish:** follow [release-1.1-checklist](docs/project/release-1.1-checklist.md). See [migration hub](docs/migration/index.md) for upgrade paths.

### Pre-release checks

```bash
./benchmarks/scripts/download.sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --exclude ontologos-conformance --exclude ontologos-contract --locked
cargo test -p ontologos-contract --release --locked
./benchmarks/scripts/check-1.0-release-gates.sh
bash scripts/ci-bindings.sh
bash scripts/ci-node.sh
./docs/scripts/check-doc-versions.sh
./docs/scripts/check-doc-snippets.sh
cargo publish -p ontologos-core --dry-run
```

Create or update [`.github/release/vX.Y.Z.md`](.github/release/) with highlights, version bumps, migration guide link, and pre-release checklist.

`cargo publish --dry-run` for downstream crates requires prior crates at the new version on crates.io (or use `cargo package -p ontologos-core --allow-dirty` per crate in publish order). On release, CI publishes in dependency order via [.github/scripts/publish-crates.sh](.github/scripts/publish-crates.sh).

Optional full local packaging check:

```bash
for crate in ontologos-core ontologos-profile ontologos-bridge ontologos-parser ontologos-rl ontologos-alc ontologos-el ontologos-dl ontologos-swrl ontologos-explain ontologos-ql ontologos-facade; do
  cargo package -p "$crate" --allow-dirty
done
```

Then:

1. Confirm workspace [Cargo.toml](Cargo.toml) `version = "1.1.0"`.
2. Ensure [CHANGELOG.md](CHANGELOG.md) has a dated `[1.1.0]` section and empty `[Unreleased]`.
3. Run `./docs/scripts/check-doc-versions.sh` and `./docs/scripts/check-doc-snippets.sh`.
4. Commit release prep on `main`.
5. Create an annotated tag: `git tag -a v1.1.0 -m "OntoLogos v1.1.0"`
6. Push commit and tag: `git push origin main && git push origin v1.1.0`
7. The [release workflow](.github/workflows/release.yml) runs when the tag is pushed (requires GitHub secrets below).
8. Create a GitHub Release from [`.github/release/v1.1.0.md`](.github/release/v1.1.0.md).

### Release secrets

| Secret | Purpose |
|--------|---------|
| `CARGO_REGISTRY_TOKEN` | Publish Rust crates to [crates.io](https://crates.io) |
| `PYPI_API_TOKEN` | Publish the `ontologos` Python package to [PyPI](https://pypi.org/project/ontologos/) |

Create a PyPI API token at https://pypi.org/manage/account/token/ (scope: entire account or project `ontologos`). Add it in the repo under **Settings → Secrets and variables → Actions**.

On each release tag, CI publishes:

- **crates.io** — crates listed in [.github/scripts/publish-crates.sh](.github/scripts/publish-crates.sh) (12 crates: `ontologos-core`, `ontologos-profile`, `ontologos-bridge`, `ontologos-parser`, `ontologos-rl`, `ontologos-alc`, `ontologos-el`, `ontologos-dl`, `ontologos-swrl`, `ontologos-explain`, `ontologos-ql`, `ontologos-facade`)
- **PyPI** — `ontologos` via release CI (`maturin-action`): Linux (x86_64, aarch64), macOS (x86_64, aarch64), Windows (x64, aarch64), plus sdist. Manual upload: [.github/scripts/publish-pypi.sh](.github/scripts/publish-pypi.sh)

- **Tags:** Release tags follow semver (`v0.9.0`, …)
- **CHANGELOG:** [Keep a Changelog](https://keepachangelog.com/) format in [CHANGELOG.md](CHANGELOG.md)

## Questions

Open a [GitHub issue](https://github.com/eddiethedean/ontologos/issues) for bugs, feature requests, or design questions. Check [FAQ.md](FAQ.md) and [Troubleshooting](docs/guides/troubleshooting.md) first.

There is no Discord or mailing list — GitHub Issues is the primary support channel.

See also [Code of Conduct](CODE_OF_CONDUCT.md) and [Security policy](SECURITY.md).
