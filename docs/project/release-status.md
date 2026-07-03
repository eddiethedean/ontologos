# Release status

--8<-- "snippets/channel-banner.md"

Single source of truth for version and distribution channels. Update this page when tagging releases.

## Current channels

| Channel | Version | Notes |
|---------|---------|-------|
| **crates.io** (library crates) | **1.0.0** (published) | Latest installable release |
| **PyPI** | **1.0.0** (published) | `pip install ontologos` |
| **Latest git tag** | **v1.0.0** | Annotated semver tags on `main` |
| **`main` branch** | **1.0.1** workspace (pre-release) | Patch fixes staged; crates.io/PyPI still **1.0.0** until **v1.0.1** publish |

Published crates (12, dependency order in `.github/scripts/publish-crates.sh`): `ontologos-core`, `ontologos-profile`, `ontologos-bridge`, `ontologos-parser`, `ontologos-rl`, `ontologos-alc`, `ontologos-el`, `ontologos-dl`, `ontologos-swrl`, `ontologos-explain`, `ontologos-ql`, `ontologos-facade`.

CLI (`ontologos-cli`), Python (`ontologos-py`), and conformance are **source-build only** — not on crates.io (Python wheels ship via PyPI).

## HermiT parity snapshot (v1.0.0, 2026-07-03)

```bash
bash benchmarks/scripts/hermit-burndown.sh status
bash benchmarks/scripts/report-conformance-coverage.sh
bash benchmarks/scripts/check-1.0-release-gates.sh
bash benchmarks/scripts/check-hermit-parity-phases.sh
```

| Metric | Value |
|--------|------:|
| Catalog `parity_pct` | **100%** (`java_planned = 0`, `wg_planned = 0`) |
| Composite `true_parity_pct` | **100%** (blocking CI) |
| `in_scope_total` | **889** |
| Active conformance tests | **1048** / **1049** total (**1** hand-written `#[ignore]` in `hermit_owllink.rs`) |
| Generated catalog `#[ignore]` | **0** (`check-hermit-ignore-budget.sh` ceiling **0**) |
| Runnable Java axiom + WG @ 30s | **450** + **428** (blocking CI, full suite) |
| Promoted IDs (`phase9_closure`) | **400** axiom + **428** WG |
| DL OFN pass rate @ 30s | **277/277** |
| Documented CE exclusions | **13** Ian/ComplexConcept + `testIanBackjumping3` (**70** `excluded` catalog cases) |
| `check-1.0-release-gates.sh` | **Green** (blocking in CI) |

Metric definitions: [Evaluator scope](../guides/evaluator-scope.md).

## Profile stability

See the canonical [Profile stability matrix](../guides/profile-stability.md). Summary:

| Area | Status |
|------|--------|
| OWL EL, RL, RDFS | **Stable** on **v1.0.0** |
| OWL DL (`--profile dl`) | **Stable** on **v1.0.0** |
| SWRL | **Stable** on **v1.0.0** |
| ALC / `dl-preview` | **Preview** |
| Python bindings, explain (EL) | **Stable** on **v1.0.0** |

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
cargo install --git https://github.com/eddiethedean/ontologos --tag v1.0.0 ontologos-cli
```

Requires **Rust 1.88+**.

## Release history

| Tag | Theme |
|-----|-------|
| [v1.0.0](https://github.com/eddiethedean/ontologos/releases/tag/v1.0.0) | HermiT parity milestone — OWL 2 DL + SWRL |
| [v0.9.0](https://github.com/eddiethedean/ontologos/releases/tag/v0.9.0) | Python ecosystem |
| [v0.8.0](https://github.com/eddiethedean/ontologos/releases/tag/v0.8.0) | Incremental reasoning |
| [v0.7.0](https://github.com/eddiethedean/ontologos/releases/tag/v0.7.0) | Bridge adapters |

Full notes: [Release notes](release-notes.md) · [CHANGELOG](changelog.md)

## Maintainer tagging

See [Contributing — Release checklist](../../CONTRIBUTING.md) for the full tag and publish workflow.
