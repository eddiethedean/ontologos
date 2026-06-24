# Release status

Single source of truth for version and distribution channels. Update this page when tagging releases.

## Current channels

| Channel | Version | Notes |
|---------|---------|-------|
| **crates.io** (library crates) | **0.9.0** (published) | Latest installable release |
| **PyPI** | **0.9.0** (published) | `pip install ontologos` |
| **Latest git tag** | **v0.9.0** | Annotated semver tags on `main` |
| **`main` branch** | **1.0.0** workspace (pre-release) | Tag **v1.0.0** pending [ROADMAP Phase 9](https://github.com/eddiethedean/ontologos/blob/main/ROADMAP.md#phase-9--v100-tag-100-in-scope-parity) (~**58%** in-scope catalog parity) |

Published crates: `ontologos-core`, `ontologos-parser`, `ontologos-profile`, `ontologos-bridge`, `ontologos-rdfs`, `ontologos-rl`, `ontologos-el`, `ontologos-query`, `ontologos-explain`, `ontologos-facade`.

CLI (`ontologos-cli`) and conformance crates are **source-build only** — not on crates.io.

## Profile stability

See the canonical [Profile stability matrix](../guides/profile-stability.md). Summary:

| Area | Status |
|------|--------|
| OWL EL, RL, RDFS | **Stable** for shipped profiles |
| OWL DL (`--profile dl`) | **Pre-release** — not production HermiT replacement |
| ALC / `dl-preview` / SWRL | **Preview** |
| Python bindings, explain (EL) | **Stable** on v0.9.0 |
| Incremental EL/RL/RDFS | **Stable** (0.8+) |

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

### Published (production today)

**Rust:**

```toml
ontologos-core = "0.9.0"
ontologos-parser = "0.9.0"
# … bump all ontologos-* crates together
```

**Python:**

```bash
pip install ontologos==0.9.0
```

### `main` branch (1.0.0 workspace)

Build from git and pin `"1.0.0"` on all workspace crates, or use `cargo install --git ...`. PyPI **1.0.0** ships only when the v1.0.0 tag is published.

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
