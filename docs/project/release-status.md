# Release status

Single source of truth for version and distribution channels. Update this page when tagging releases.

## Current channels

| Channel | Version | Notes |
|---------|---------|-------|
| **crates.io** (library crates) | **0.9.0** | `ontologos-core`, `ontologos-parser`, `ontologos-profile`, `ontologos-bridge`, `ontologos-rdfs`, `ontologos-rl`, `ontologos-el`, `ontologos-query`, `ontologos-explain`, `ontologos-facade` |
| **PyPI** | **0.9.0** | `pip install ontologos` |
| **Latest git tag** | **v0.9.0** | Annotated semver tags on `main` |
| **`main` branch** | **0.9.0** workspace + preview DL/ALC | May include doc/fix commits ahead of tags |

CLI (`ontologos-cli`) and conformance crates are **source-build only** — not on crates.io.

## What is stable vs preview

| Area | Status |
|------|--------|
| OWL EL, RL, RDFS classification | **Stable** (0.9.0) |
| Python bindings, explain (EL) | **Stable** (0.9.0) |
| Incremental EL/RL/RDFS | **Stable** (0.8+) |
| DL / ALC / SWRL profiles | **Preview** on `main` — see [Preview profiles](../guides/preview-profiles.md) |
| Full HermiT OWL DL parity | **Planned 1.0** — see [Roadmap summary](roadmap-summary.md) |

## Install pins

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

**CLI (from git):**

```bash
cargo install --git https://github.com/eddiethedean/ontologos ontologos-cli
```

Requires **Rust 1.88+**.

## Release history

| Tag | Theme |
|-----|-------|
| [v0.9.0](https://github.com/eddiethedean/ontologos/releases/tag/v0.9.0) | Python ecosystem |
| [v0.8.0](https://github.com/eddiethedean/ontologos/releases/tag/v0.8.0) | Incremental reasoning |
| [v0.7.0](https://github.com/eddiethedean/ontologos/releases/tag/v0.7.0) | Bridge adapters |

Full notes: [Release notes](release-notes.md) · [CHANGELOG](changelog.md)

## Maintainer tagging

See [Contributing — Release checklist](../../CONTRIBUTING.md) for the full tag and publish workflow.
