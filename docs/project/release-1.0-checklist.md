# v1.0.0 release checklist (Tier A)

Engineering gates are **green on `main`**. This checklist completes **publish + tag** (manual maintainer steps).

## Pre-flight (local)

```bash
bash benchmarks/scripts/check-1.0-release-gates.sh
bash benchmarks/scripts/check-hermit-parity-phases.sh
cargo clippy --workspace --all-targets -- -D warnings
cargo doc -p ontologos-dl --no-deps
cargo doc -p ontologos-alc --no-deps
```

## Publish

1. Confirm [Cargo.toml](../../Cargo.toml) workspace `version = "1.0.0"`.
2. Update [CHANGELOG.md](../../CHANGELOG.md) with v1.0.0 section.
3. Push annotated tag **`v1.0.0`** — triggers [.github/workflows/release.yml](../../.github/workflows/release.yml):
   - `verify` (full conformance @ 30s)
   - `publish-crates` (all `publish = true` crates including `ontologos-dl`, `ontologos-alc`, `ontologos-swrl`, `ontologos-abox`, `ontologos-ql`)
   - PyPI maturin wheels (`ontologos==1.0.0`)

## Post-publish docs

- [release-status.md](release-status.md) — channels table → **1.0.0 published**
- [profile-stability.md](../guides/profile-stability.md) — `dl` → **Stable**
- [README.md](../../README.md) — default pins `1.0.0`
- [docs/internal/roadmap.md](https://github.com/eddiethedean/ontologos/blob/main/docs/internal/roadmap.md) — Phase 9 publish boxes checked

## Verify install

```bash
cargo install ontologos-dl --version 1.0.0
pip install ontologos==1.0.0
ontologos classify --profile dl family.owl
```

## Beyond Tier A

Literal catalog (1019/1019), strict taxonomy, and performance targets are tracked in [parity-roadmap.md](../internal/parity-roadmap.md).
