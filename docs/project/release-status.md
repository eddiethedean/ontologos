# Release status

Single source of truth for version and distribution channels. Update this page when tagging releases.

## Current channels

| Channel | Version | Notes |
|---------|---------|-------|
| **crates.io** (library crates) | **0.9.0** (published) · **1.0.0** (workspace) | `ontologos-core`, `ontologos-parser`, `ontologos-profile`, `ontologos-bridge`, `ontologos-rdfs`, `ontologos-rl`, `ontologos-el`, `ontologos-query`, `ontologos-explain`, `ontologos-facade` |
| **PyPI** | **0.9.0** (published) · **1.0.0** (workspace) | `pip install ontologos` |
| **Latest git tag** | **v0.9.0** | Annotated semver tags on `main` |
| **`main` branch** | **1.0.0** workspace | Tag **v1.0.0** pending [ROADMAP Phase 9](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md#phase-9--v100-tag-100-in-scope-parity) (~**58%** in-scope catalog parity); `check-1.0-release-gates.sh` passes at **593** active tests but is **not sufficient** alone |

CLI (`ontologos-cli`) and conformance crates are **source-build only** — not on crates.io.

## What is stable vs preview

| Area | Status |
|------|--------|
| OWL EL, RL, RDFS classification | **Stable** for shipped profiles |
| OWL DL (`--profile dl`) | **Stable for gated corpora** (`family.owl` in CI; `pizza.owl` / `go-subset.owl` optional slow gates) — **not** production HermiT replacement |
| Python bindings, explain (EL + DL smoke) | **Stable** (1.0.0) |
| Incremental EL/RL/RDFS | **Stable** (0.8+) |
| ALC / `dl-preview` | **Preview** — explicit gating and subset checks |
| Full HermiT OWL DL parity | **In progress** — [ROADMAP parity phases](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md#hermit-parity-phases-path-to-v100-tag) Phase 3; [gap report](https://github.com/eddiethedean/ontologos/blob/main/docs/internal/hermit-parity-gap-report.md) |

**Production OWL DL:** use Protégé + HermiT or Konclude ([FAQ](../../FAQ.md)).

## Conformance snapshot (live)

```bash
bash benchmarks/scripts/report-ci-gate-status.sh
bash benchmarks/scripts/report-conformance-coverage.sh
bash benchmarks/scripts/check-1.0-release-gates.sh
bash benchmarks/scripts/check-hermit-parity-phases.sh   # fails until Phase 9 (100% catalog parity)
```

Current inventory: **593** active / **1063** total conformance tests; **~58%** in-scope catalog parity (`parity_pct`). CI runs release gates on every PR (informational); **v1.0.0 tag** requires Phase 9 + blocking CI.

## Install pins

**Rust:**

```toml
ontologos-core = "1.0.0"
ontologos-parser = "1.0.0"
# … bump all ontologos-* crates together
```

**Python:**

```bash
pip install ontologos==1.0.0
```

**CLI (from git):**

```bash
cargo install --git https://github.com/eddiethedean/ontologos ontologos-cli
```

Requires **Rust 1.88+**.

## Release history

| Tag | Theme |
|-----|-------|
| v1.0.0 (pending) | HermiT parity milestone |
| [v0.9.0](https://github.com/eddiethedean/ontologos/releases/tag/v0.9.0) | Python ecosystem |
| [v0.8.0](https://github.com/eddiethedean/ontologos/releases/tag/v0.8.0) | Incremental reasoning |
| [v0.7.0](https://github.com/eddiethedean/ontologos/releases/tag/v0.7.0) | Bridge adapters |

Full notes: [Release notes](release-notes.md) · [CHANGELOG](changelog.md)

## Maintainer tagging

See [Contributing — Release checklist](../../CONTRIBUTING.md) for the full tag and publish workflow.
