# Post-1.0.0 documentation update

Runbook for maintainers when the annotated **v1.0.0** tag is published to crates.io and PyPI. Execute in order; verify with the scripts at the end.

## 1. Version pins

| Location | Action |
|----------|--------|
| Workspace [`Cargo.toml`](../../Cargo.toml) | Already `1.0.0` before tag |
| [`crates/ontologos-py/pyproject.toml`](../../crates/ontologos-py/pyproject.toml) | Bump to `1.0.0` |
| [`crates/ontologos-py/python/ontologos/__init__.py`](../../crates/ontologos-py/python/ontologos/__init__.py) | Bump `__version__` |
| [`docs/scripts/check-doc-versions.sh`](../scripts/check-doc-versions.sh) | Set `PUBLISHED_VERSION="1.0.0"` |
| Published-pin files (see script) | Replace `0.9.0` → `1.0.0` in install blocks |
| [`docs/snippets/channel-banner.md`](../snippets/channel-banner.md) | Update to single-channel messaging |
| [`README.md`](../../README.md) | Lead with v1.0.0 as published; archive 0.9.0 migration links |
| [`FAQ.md`](../../FAQ.md) | Update "which version" table |

## 2. docs.rs links

Update all `https://docs.rs/ontologos-*/0.9.0` → `1.0.0` in:

- [`mkdocs.yml`](../../mkdocs.yml) (Reference → Rust API section)
- [`docs/getting-started/`](.), [`docs/guides/`](.), [`docs/reference/`](.), [`docs/examples/`](../examples/)
- [`README.md`](../../README.md), [`FAQ.md`](../../FAQ.md)

Or run a repo-wide replace after updating `PUBLISHED_VERSION` in `check-doc-versions.sh`.

## 3. Channel banners and hero copy

| Page | Action |
|------|--------|
| [`docs/index.md`](../index.md) | Hero badges: `v1.0.0 published`; remove pre-release badge |
| [`docs/guides/install-channels.md`](../guides/install-channels.md) | Collapse to single published channel; move 0.9.0 to migration |
| [`docs/snippets/channel-banner.md`](../snippets/channel-banner.md) | Simplify or remove if single channel |
| [`docs/migration/index.md`](../migration/index.md) | Point "published today" to v1.0.0 |
| [`docs/project/release-status.md`](release-status.md) | Update channel table |

## 4. Profile stability

- [`docs/guides/profile-stability.md`](../guides/profile-stability.md) — mark `dl` and `swrl` stable on published 1.0.0
- Remove "not on PyPI" caveats from Python guide and CLI reference
- [`docs/comparison.md`](../comparison.md) — update production-ready rows

## 5. Migration hub

- [`docs/migration/index.md`](../migration/index.md) — published = v1.0.0
- Add note that v0.9.x users should follow [v0.9.x → v1.0.0](v0.9.x-to-v1.0.0.md)
- [`CHANGELOG.md`](../../CHANGELOG.md) — move `[Unreleased]` into dated `[1.0.0]` if not already

## 6. CLI and PyPI

- [`crates/ontologos-cli/src/main.rs`](../../crates/ontologos-cli/src/main.rs) — `after_help` version (checked by `check-doc-versions.sh`)
- GitHub Release from [`.github/release/v1.0.0.md`](../../.github/release/) (create if missing)
- PyPI wheels via release CI

## 7. Verification

```bash
./benchmarks/scripts/download.sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
./docs/scripts/check-doc-versions.sh
./docs/scripts/check-doc-snippets.sh
./docs/build-site.sh
```

## 8. Optional follow-up

- Archive dual-channel language in historical migration guides
- Update PyPI / crates.io crate READMEs on next publish
- Announce on GitHub Release with link to [Evaluator scope](../guides/evaluator-scope.md)

## Related

- [Release 1.0 checklist](release-1.0-checklist.md)
- [Release status](release-status.md)
- [CONTRIBUTING.md](../../CONTRIBUTING.md) — release section
