# Release status

--8<-- "snippets/channel-banner.md"

Single source of truth for version and distribution channels. Update this page when tagging releases.

## Current channels

| Channel | Version | Notes |
|---------|---------|-------|
| **crates.io** (library crates) | **0.9.0** (published) | Latest installable release |
| **PyPI** | **0.9.0** (published) | `pip install ontologos` |
| **Latest git tag** | **v0.9.0** | Annotated semver tags on `main` |
| **`main` branch** | **1.0.0** workspace (pre-release) | Engineering gates green; **v1.0.0 tag** pending crates.io + PyPI publish |

Published crates (12, dependency order in `.github/scripts/publish-crates.sh`): `ontologos-core`, `ontologos-profile`, `ontologos-bridge`, `ontologos-parser`, `ontologos-rl`, `ontologos-alc`, `ontologos-el`, `ontologos-dl`, `ontologos-swrl`, `ontologos-explain`, `ontologos-ql`, `ontologos-facade`.

CLI (`ontologos-cli`), Python (`ontologos-py`), conformance, and watch are **source-build only** — not on crates.io.

## HermiT parity snapshot (`main`, 2026-06-29)

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
| Active conformance tests | **1009** / **1152** total (**143** `#[ignore]`) |
| Runnable Java axiom + WG @ 30s | **450** + **428** (blocking CI, full suite) |
| Promoted IDs (`phase9_closure`) | **400** axiom + **428** WG |
| DL OFN pass rate @ 30s | **277/277** |
| Documented CE exclusions | **13** Ian/ComplexConcept + `testIanBackjumping3` (**70** `excluded` catalog cases) |
| `check-1.0-release-gates.sh` | **Green** (blocking in CI) |

**Remaining for v1.0.0 publish:** architecture review prep complete on `main` — follow [release-1.0-checklist.md](release-1.0-checklist.md) (annotated tag `v1.0.0` triggers crates.io + PyPI). After publish: [Post-1.0 doc update](post-1.0-doc-update.md). Metric definitions: [Evaluator scope](../guides/evaluator-scope.md).

## Profile stability

See the canonical [Profile stability matrix](../guides/profile-stability.md). Summary:

| Area | Status |
|------|--------|
| OWL EL, RL, RDFS | **Stable** on published **v0.9.0** |
| OWL DL (`--profile dl`) on **`main`** | **Stable** (workspace) — publish pending; see [release checklist](release-1.0-checklist.md) |
| ALC / `dl-preview` | **Preview** |
| SWRL | **Stable** on workspace **1.0.0**; **not on PyPI 0.9.0** |
| Python bindings, explain (EL) | **Stable** on v0.9.0 |

**Production EL/RL/RDFS today:** use **v0.9.0** pins on crates.io/PyPI. **Production OWL DL and SWRL:** build from **`main`** until v1.0.0 publishes to crates.io/PyPI (`profile="dl"`). See [FAQ](faq.md).

## Conformance snapshot (live)

```bash
bash benchmarks/scripts/report-ci-gate-status.sh
bash benchmarks/scripts/report-conformance-coverage.sh
bash benchmarks/scripts/check-1.0-release-gates.sh
bash benchmarks/scripts/check-hermit-parity-phases.sh
```

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
| v1.0.0 (pending) | HermiT parity milestone — publish + tag |
| [v0.9.0](https://github.com/eddiethedean/ontologos/releases/tag/v0.9.0) | Python ecosystem |
| [v0.8.0](https://github.com/eddiethedean/ontologos/releases/tag/v0.8.0) | Incremental reasoning |
| [v0.7.0](https://github.com/eddiethedean/ontologos/releases/tag/v0.7.0) | Bridge adapters |

Full notes: [Release notes](release-notes.md) · [CHANGELOG](changelog.md)

## Maintainer tagging

See [Contributing — Release checklist](../../CONTRIBUTING.md) for the full tag and publish workflow.
