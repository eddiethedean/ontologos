# Release status

--8<-- "snippets/channel-banner.md"

**Single source of truth** for version and distribution channels. Update this page when tagging releases.

## Current channels

| Channel | Version | Notes |
|---------|---------|-------|
| **crates.io** (12 library crates) | **1.1.3** | Publish with annotated **v1.1.3** tag |
| **PyPI** | **1.1.3** | `pip install ontologos` |
| **docs.rs** | **1.1.3** | Matches crates.io |
| **Latest git tag** | **v1.1.3** | Annotated release on GitHub |
| **`main` branch** | **1.1.3** workspace | Multi-language bindings + shared FFI |

Published library crates (12, dependency order in `.github/scripts/publish-crates.sh`): `ontologos-core`, `ontologos-profile`, `ontologos-bridge`, `ontologos-parser`, `ontologos-rl`, `ontologos-alc`, `ontologos-el`, `ontologos-dl`, `ontologos-swrl`, `ontologos-explain`, `ontologos-ql`, `ontologos-facade`.

**Source-build / PyPI wheels:** CLI (`ontologos-cli`), Python (`ontologos-py`), Node (`ontologos-node`), WASM (`ontologos-wasm`), Java (`ontologos-jni`), .NET (`ontologos-dotnet`), C/C++ (`ontologos-c` + `ontologos-ffi`).

## What version am I running?

| Surface | Command |
|---------|---------|
| **Python** | `python -c "import ontologos; print(ontologos.__version__)"` |
| **CLI** | `ontologos --version` |
| **Rust dependency** | `cargo tree -p ontologos-core \| head -1` |
| **crates.io latest** | [crates.io/crates/ontologos-core](https://crates.io/crates/ontologos-core) |
| **PyPI latest** | [pypi.org/project/ontologos](https://pypi.org/project/ontologos/) |

Expected: **1.1.3** from registries after publish.

## v1.1.3 highlights

| Area | What's fixed |
|------|--------------|
| **Conformance** | Drop weak IRI-shape entailment guard for Consistent-but-all-unsat; defer WG case until DL ⊥ proof |
| **Tests** | Oracle-backed asserts; shared `semantic-fixtures.json`; triage demotion |

Drop-in patch over **v1.1.2** — bump all `ontologos-*` pins together. See [CHANGELOG](../../CHANGELOG.md).

## v1.1.2 highlights

| Area | What's fixed |
|------|--------------|
| **Docs** | Restore `v1.0.x-to-v1.1.0` migration guide links (MkDocs strict build) |

Drop-in patch over **v1.1.1** — bump all `ontologos-*` pins together. See [CHANGELOG](../../CHANGELOG.md).

## v1.1.1 highlights

| Area | What's fixed |
|------|--------------|
| **Parser** | Lenient import kind-conflict handling; dangling DL expression validation |
| **DL** | Malformed XSD numerics; `from_json_with_limits` validation |
| **SWRL** | EL taxonomy dependency for class-variable rule bodies |
| **RL / EL / bindings** | Materialization clash reporting; `Profile::Auto` incremental EL; Python/JS query rewrite |

Drop-in patch over **v1.1.0**. See [CHANGELOG](../../CHANGELOG.md).

## v1.1.0 highlights

| Area | What's new |
|------|------------|
| **Shared FFI** | `ontologos-ffi` — stable C ABI for native bindings |
| **Java** | JNI + Maven (`dev.ontologos:ontologos`) — source-build |
| **.NET** | P/Invoke + C# API — source-build |
| **C/C++** | `libontologos_c` + headers — source-build |
| **Node / WASM** | N-API and wasm-pack over `ontologos-js` |
| **CI** | `scripts/ci-bindings.sh`, `scripts/ci-node.sh` |

See [v1.0.x → v1.1.0 migration](../migration/v1.0.x-to-v1.1.0.md).

## Install pins

**Rust:**

```toml
ontologos-core = "1.1.3"
ontologos-parser = "1.1.3"
ontologos-facade = "1.1.3"
# … bump all ontologos-* crates together
```

**Python:**

```bash
pip install ontologos
# or pin explicitly:
pip install ontologos==1.1.3
```

**CLI (from git — not on crates.io):**

```bash
cargo install --git https://github.com/eddiethedean/ontologos --tag v1.1.3 ontologos-cli
```

Requires **Rust 1.88+**.

## HermiT parity snapshot (2026-07-04)

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

Metric definitions: [Evaluator scope](../guides/evaluator-scope.md).

## Profile stability

See the canonical [Profile stability matrix](../guides/profile-stability.md). Summary:

| Area | Status |
|------|--------|
| OWL EL, RL, RDFS | **Stable** on **v1.1.3** |
| OWL DL (`--profile dl`) | **Stable** — validate on your corpus |
| SWRL | **Stable** |
| ALC / `dl-preview` | **Preview** |
| Python | **Stable** on PyPI |
| Node, Java, .NET, C/C++, WASM | **Stable** (source-build) |

## Release history

| Tag | Theme |
|-----|-------|
| [v1.1.3](https://github.com/eddiethedean/ontologos/releases/tag/v1.1.3) | Docs migration link fixes |
| [v1.1.1](https://github.com/eddiethedean/ontologos/releases/tag/v1.1.1) | Parser, DL, SWRL, RL, and binding bug fixes |
| [v1.1.0](https://github.com/eddiethedean/ontologos/releases/tag/v1.1.0) | Multi-language bindings (Java, .NET, C/C++, shared FFI) |
| [v1.0.0](https://github.com/eddiethedean/ontologos/releases/tag/v1.0.0) | HermiT parity milestone — OWL 2 DL + SWRL |
| [v0.9.0](https://github.com/eddiethedean/ontologos/releases/tag/v0.9.0) | Python ecosystem |

Full notes: [Release notes](release-notes.md) · [CHANGELOG](changelog.md)

## Maintainer tagging

See [Contributing — Release checklist](../../CONTRIBUTING.md) and [v1.1.0 release checklist](release-1.1-checklist.md).
