# v1.1.1 release checklist

Pre-tag verification for **OntoLogos v1.1.1** (multi-language bindings). Execute in order.

## 1. Version alignment

| Location | Expected |
|----------|----------|
| Workspace [`Cargo.toml`](../../Cargo.toml) | `version = "1.1.1"` |
| [`crates/ontologos-py/pyproject.toml`](../../crates/ontologos-py/pyproject.toml) | `version = "1.1.1"` |
| [`crates/ontologos-py/python/ontologos/__init__.py`](../../crates/ontologos-py/python/ontologos/__init__.py) | `__version__ = "1.1.1"` |
| [`crates/ontologos-cli/src/main.rs`](../../crates/ontologos-cli/src/main.rs) | `after_help` advertises `v1.1.1` |
| [`docs/scripts/check-doc-versions.sh`](../scripts/check-doc-versions.sh) | `PUBLISHED_VERSION="1.1.1"` |

## 2. Documentation

| Location | Action |
|----------|--------|
| [channel banner (repo)](https://github.com/eddiethedean/ontologos/blob/main/docs/snippets/channel-banner.md) | Single-channel **v1.1.1** on crates.io/PyPI |
| [`docs/project/release-status.md`](release-status.md) | Published **1.1.1**; remove staged language |
| [README (repo)](https://github.com/eddiethedean/ontologos/blob/main/README.md) | Install pins **1.1.1**; CLI `--tag v1.1.1` |
| [CHANGELOG (repo)](https://github.com/eddiethedean/ontologos/blob/main/CHANGELOG.md) | Remove STAGED note from `[1.1.1]` section |
| [`.github/release/v1.1.1.md`](https://github.com/eddiethedean/ontologos/blob/main/.github/release/v1.1.1.md) | GitHub Release body ready |

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

**Full release verify** (includes ~26 min conformance — matches [release.yml](../../.github/workflows/release.yml)):

```bash
cargo test -p ontologos-conformance --release --locked
```

## 4. Tag and publish

1. Confirm [CHANGELOG](changelog.md) `[1.1.1]` section is complete; `[Unreleased]` empty.
2. Commit release prep on `main`.
3. Annotated tag:

   ```bash
   git tag -a v1.1.1 -m "OntoLogos v1.1.1"
   git push origin main && git push origin v1.1.1
   ```

4. [Release workflow](../../.github/workflows/release.yml) publishes crates.io (12 crates) + PyPI wheels on tag push.
5. Create GitHub Release from [v1.1.1 release notes](https://github.com/eddiethedean/ontologos/blob/main/.github/release/v1.1.1.md).

## 5. Secrets required

| Secret | Purpose |
|--------|---------|
| `CARGO_REGISTRY_TOKEN` | crates.io publish |
| `PYPI_API_TOKEN` | PyPI `ontologos` wheels |

## 6. Post-publish verification

```bash
pip install ontologos==1.1.1
python -c "import ontologos; assert ontologos.__version__ == '1.1.1'"
cargo install ontologos-core --version 1.1.1
```

## Related

- [Release status](release-status.md)
- [v1.0.x → v1.1.1 migration](../migration/v1.0.x-to-v1.1.1.md)
- [Contributing](contributing.md) — release section
