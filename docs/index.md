# OntoLogos documentation

<div class="ol-hero" markdown="0">
  <div class="ol-hero-badges">
    <span class="ol-badge ol-badge--accent">v1.1.4</span>
    <span class="ol-badge">Rust 1.88+</span>
    <span class="ol-badge">Python · Rust · Wasmer</span>
  </div>
  <p class="ol-hero-kicker">OntoLogos documentation</p>
  <p class="ol-hero-title">Embed OWL reasoning in your stack</p>
  <p class="ol-lead">Load OWL files, detect profiles, and classify or materialize in-process — no JVM. <strong>Rust</strong> and <strong>Python</strong> on crates.io/PyPI; <strong>WASM</strong> on <a href="https://wasmer.io/eddiethedean/ontologos">Wasmer</a>; Node, Java, .NET, and C/C++ via <a href="guides/bindings-overview.html">source build</a>.</p>
  <div class="ol-hero-actions">
    <a class="ol-hero-cta" href="getting-started/">Try in 5 minutes →</a>
    <a class="ol-hero-cta ol-hero-cta--secondary" href="guides/python/">Python quickstart</a>
    <a class="ol-hero-cta ol-hero-cta--secondary" href="guides/bindings-overview/">Bindings overview</a>
  </div>
  <p style="margin-top:1rem;margin-bottom:0"><a href="guides/install-channels/">Install channels</a> · <a href="project/release-status/">Release status</a> · <a href="guides/before-you-integrate/">Before you integrate</a> · <a href="guides/start-here/">Start here</a></p>
</div>

--8<-- "snippets/channel-banner.md"

Pick the path that matches how you work:

<div class="grid cards" markdown>

-   :material-language-rust: **Rust (crates.io)**

    ---

    Download `family.owl`, add `ontologos-core`, `ontologos-parser`, and `ontologos-rl`, run RDFS materialization—no clone required.

    [:octicons-arrow-right-24: Five-minute guide](getting-started/index.md#cratesio-only-no-clone)

-   :material-language-python: **Python**

    ---

    `pip install ontologos` — load OWL, classify with `profile="auto"`, incremental mutations.

    [:octicons-arrow-right-24: Python guide](guides/python.md)

-   :material-console: **CLI** *(git install)*

    ---

    `classify`, `materialize`, `explain` — install from git (not crates.io).

    [:octicons-arrow-right-24: CLI installation](getting-started/cli-install.md)

-   :material-magnify: **Evaluate**

    ---

    Compare vs ELK, reasonable, and HermiT fixtures; 30-minute playbook. **No Rust required:** `pip install ontologos` + `family.owl`.

    [:octicons-arrow-right-24: Evaluator playbook](guides/evaluator-playbook.md)

</div>

!!! tip "No clone required for most users"
    Use **crates.io**, **PyPI**, or [Wasmer](https://wasmer.io/eddiethedean/ontologos). Clone to contribute, run benchmarks, build Node/Java/.NET/C bindings or WASM JS glue, or build the CLI.

<div class="ol-callout" markdown="0">
  <strong>Rust 1.88+</strong> for library users — see the <a href="guides/prerequisites.html">Prerequisites decision table</a>.
</div>

!!! warning "Before you integrate"
    Read [Before you integrate](guides/before-you-integrate.md) — partial OWL mapping, import limits, and axiom count semantics.

!!! tip "Rust integrators"
    See the [Rust integration contract](guides/rust-integration-contract.md) — one page for load, classify, and consistency rules.

!!! note "Preview profiles"
    **`alc`** and **`dl-preview`** are experimental. Production OWL DL uses **`profile="dl"`**. See [Profile stability](guides/profile-stability.md).

## What you need

| Channel | Link |
|---------|------|
| **Docs** | [ontologos.readthedocs.io](https://ontologos.readthedocs.io/en/latest/) |
| **crates.io** | [ontologos-core](https://crates.io/crates/ontologos-core) and siblings |
| **PyPI** | [`pip install ontologos`](https://pypi.org/project/ontologos/) |
| **Wasmer** | [`eddiethedean/ontologos`](https://wasmer.io/eddiethedean/ontologos) |
| **Rust API** | [docs.rs/ontologos-core](https://docs.rs/ontologos-core/1.1.4) |
| **Changelog** | [project/changelog](project/changelog.md) · [GitHub](https://github.com/eddiethedean/ontologos/blob/main/CHANGELOG.md) |

## Documentation map {#documentation-map}

| Area | Start here |
|------|------------|
| New users | [Before you integrate](guides/before-you-integrate.md) · [Install channels](guides/install-channels.md) · [Start here](guides/start-here.md) |
| Examples | [Examples gallery](examples/index.md) |
| API choice | [Choosing an API](guides/choosing-an-api.md) · [Profile stability](guides/profile-stability.md) |
| Evaluate | [Evaluator playbook](guides/evaluator-playbook.md) · [Evaluator scope](guides/evaluator-scope.md) · [Comparison](comparison.md) |
| Upgrade | [Migration hub](migration/index.md) · [Release status](project/release-status.md) |

## Learning path

**New to OWL?** Start with the [Glossary](guides/glossary.md) and [When not to use OntoLogos](guides/when-not-to-use.md), then [Python guide](guides/python.md) or [Rust quickstart](getting-started/index.md#cratesio-only-no-clone).

**Know OWL already:**

1. [Before you integrate](guides/before-you-integrate.md)
2. [Getting started — crates.io quick start](getting-started/index.md#cratesio-only-no-clone)
3. [Classify in five minutes](getting-started/classify-quickstart.md)
4. [Load an OWL file](getting-started/load-owl-file.md)
5. Profile guides: [RDFS](getting-started/rdfs-materialization.md) · [RL](getting-started/owl-rl-saturation.md) · [EL](getting-started/owl-el-classification.md) · [SWRL](getting-started/swrl.md)
6. [Profile detection](guides/profile-detection.md)
7. [Error reference](reference/errors.md)

## Capability matrix (v1.1.4)

| Capability | Library | CLI | Python |
|------------|---------|-----|--------|
| Load OWL files | Yes | Yes | Yes |
| Profile detection | Yes | `profile` | `"auto"` |
| RDFS materialization | Yes | `materialize` | `profile="rdfs"` |
| OWL RL saturation | Yes | `classify --profile rl` | `profile="rl"` |
| OWL EL taxonomy | Yes | `classify --profile el` | `profile="el"` |
| OWL DL | Yes | `classify --profile dl` | `profile="dl"` |
| SWRL | Yes | `classify --profile swrl` | `profile="swrl"` |
| Incremental reasoning | Yes | `--incremental` | `incremental=True` |
| Explanations | Yes | `explain` | `explain()` |
| Taxonomy DataFrame export | No | No | Yes (pandas/polars) |
