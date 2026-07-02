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

**Architecture review prep (done on `main`):**

- [x] Doc/code drift: `security.md`, `preview-profiles.md`, entailment cap 192, facade-only classify
- [x] Production DL guide: `docs/guides/production-integration.md#owl-dl-in-production`
- [x] Parser concurrency documented in `docs/security.md`
- [x] Tier C strict identity gate waived to informational (`compare-tier-c-strict-family.sh`)
- [x] `ClassifyOutcome` marked `#[non_exhaustive]`
- [x] `SECURITY.md` supported versions → 1.0.x

## Publish

1. Confirm [Cargo.toml](../../Cargo.toml) workspace `version = "1.0.0"`.
2. Update [CHANGELOG.md](../../CHANGELOG.md) with v1.0.0 section (merge `[Unreleased]` if needed).
3. Push annotated tag **`v1.0.0`** — triggers [.github/workflows/release.yml](../../.github/workflows/release.yml):
   - `verify` (full conformance @ 30s)
   - `publish-crates` (all `publish = true` crates including `ontologos-dl`, `ontologos-alc`, `ontologos-swrl`, `ontologos-ql`)
   - PyPI maturin wheels (`ontologos==1.0.0`)

```bash
git tag -a v1.0.0 -m "OntoLogos v1.0.0"
git push origin v1.0.0
```

Dry-run (optional, before tag):

```bash
cargo publish -p ontologos-core --dry-run
```

## Post-publish docs

- [release-status.md](release-status.md) — channels table → **1.0.0 published**
- [profile-stability.md](../guides/profile-stability.md) — `dl` → **Stable**
- [README.md](../../README.md) — default pins `1.0.0`
- [post-1.0-doc-update.md](post-1.0-doc-update.md) — full doc pin migration runbook
- [docs/internal/roadmap.md](https://github.com/eddiethedean/ontologos/blob/main/docs/internal/roadmap.md) — Phase 9 publish boxes checked

## Verify install

```bash
cargo new ontologos-dl-smoke && cd ontologos-dl-smoke
cargo add ontologos-dl@1.0.0 ontologos-parser@1.0.0 ontologos-facade@1.0.0
pip install ontologos==1.0.0
cargo install --git https://github.com/eddiethedean/ontologos --tag v1.0.0 ontologos-cli
ontologos classify --profile dl family.owl
```

## Beyond Tier A

Literal catalog (1019/1019), strict taxonomy identity (`--max-extra 0`), and performance targets are tracked in [parity-roadmap.md](../internal/parity-roadmap.md). Strict identity is **informational** — sound superset is the release contract.
