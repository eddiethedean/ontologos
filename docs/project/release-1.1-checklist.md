# v1.1.4 release checklist

Pre-tag verification for **OntoLogos v1.1.4** (conformance nightly deferred-WG fix). Execute in order.

## 1. Version alignment

| Location | Expected |
|----------|----------|
| Workspace [`Cargo.toml`](../../Cargo.toml) | `version = "1.1.4"` |
| [`crates/ontologos-py/pyproject.toml`](../../crates/ontologos-py/pyproject.toml) | `version = "1.1.4"` |
| [`crates/ontologos-py/python/ontologos/__init__.py`](../../crates/ontologos-py/python/ontologos/__init__.py) | `__version__ = "1.1.4"` |
| [`crates/ontologos-cli/src/main.rs`](../../crates/ontologos-cli/src/main.rs) | `after_help` advertises `v1.1.4` |
| [`docs/scripts/check-doc-versions.sh`](../scripts/check-doc-versions.sh) | `PUBLISHED_VERSION="1.1.4"` |
| [`.github/release/v1.1.4.md`](../../.github/release/v1.1.4.md) | GitHub Release body ready |

## 2. Documentation

| Location | Action |
|----------|--------|
| [channel banner (repo)](https://github.com/eddiethedean/ontologos/blob/main/docs/snippets/channel-banner.md) | Single-channel **v1.1.4** on crates.io/PyPI |
| [`docs/project/release-status.md`](release-status.md) | Published **1.1.4** |
| [README (repo)](https://github.com/eddiethedean/ontologos/blob/main/README.md) | Install pins **1.1.4**; CLI `--tag v1.1.4` |
| [CHANGELOG (repo)](https://github.com/eddiethedean/ontologos/blob/main/CHANGELOG.md) | Dated `[1.1.4]` section; empty `[Unreleased]` |
| [`.github/release/v1.1.4.md`](https://github.com/eddiethedean/ontologos/blob/main/.github/release/v1.1.4.md) | GitHub Release body ready |

Run:

```bash
./docs/scripts/check-doc-versions.sh
./docs/scripts/check-doc-snippets.sh
./docs/build-site.sh
```

## 3. Pre-release CI (local)

```bash
./benchmarks/scripts/download.sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --exclude ontologos-conformance --exclude ontologos-contract --locked
cargo test -p ontologos-contract --release --locked
./benchmarks/scripts/check-hermit-parity-phases.sh
./benchmarks/scripts/check-1.0-release-gates.sh
bash scripts/ci-bindings.sh
bash scripts/ci-node.sh
cargo build -p ontologos-cli --release
```

**Full release verify** (includes ~26 min conformance — matches [release.yml](https://github.com/eddiethedean/ontologos/blob/main/.github/workflows/release.yml)):

```bash
export ONTOLOGOS_DL_BUDGET_SECS=30 ONTOLOGOS_WG_SHORTCUTS=1 ONTOLOGOS_CONFORMANCE=1
cargo test --workspace --locked
cargo test -p ontologos-conformance --release --locked
```

Note: `Consistent-but-all-unsat` is `deferred` / `#[ignore]` in 1.1.4 (weak IRI guard removed; DL does not yet prove named-class ⊥). Re-promote when `named_classes_unsatisfiable` succeeds.

## 4. Tag and publish

1. Confirm [CHANGELOG](https://github.com/eddiethedean/ontologos/blob/main/CHANGELOG.md) `[1.1.4]` section is complete; `[Unreleased]` empty.
2. Commit release prep on `main`.
3. Annotated tag:

   ```bash
   git tag -a v1.1.4 -m "OntoLogos v1.1.4"
   git push origin main && git push origin v1.1.4
   ```

4. [Release workflow](https://github.com/eddiethedean/ontologos/blob/main/.github/workflows/release.yml) publishes crates.io (12 crates) + PyPI wheels on tag push.
5. Create GitHub Release from [v1.1.4 release notes](https://github.com/eddiethedean/ontologos/blob/main/.github/release/v1.1.4.md).

## 5. Secrets required

| Secret | Purpose |
|--------|---------|
| `CARGO_REGISTRY_TOKEN` | crates.io publish |
| `PYPI_API_TOKEN` | PyPI `ontologos` wheels |

## 6. Post-publish verification

```bash
pip install ontologos==1.1.4
python -c "import ontologos; assert ontologos.__version__ == '1.1.4'"
cargo install ontologos-core --version 1.1.4
```

## Related

- [Release status](release-status.md)
- [v1.0.x → v1.1.0 migration](../migration/v1.0.x-to-v1.1.0.md)
- [Contributing](https://github.com/eddiethedean/ontologos/blob/main/CONTRIBUTING.md) — release section
- [Test oracle policy](https://github.com/eddiethedean/ontologos/blob/main/docs/internal/test-oracle-policy.md)
