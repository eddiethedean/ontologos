# OntoLogos documentation

<div class="ol-hero" markdown="0">
  <div class="ol-hero-badges">
    <span class="ol-badge ol-badge--accent">v0.9.0 published</span>
    <span class="ol-badge">main = 1.0.0 pre-release</span>
    <span class="ol-badge">Rust 1.88+</span>
  </div>
  <p class="ol-hero-kicker">OntoLogos documentation</p>
  <p class="ol-hero-title">Native Rust ontology reasoning</p>
  <p class="ol-lead">Load OWL files, detect profiles, and run RDFS materialization, OWL RL saturation, OWL EL taxonomy, and in-progress OWL DL—through stable Rust facades, CLI, and Python bindings.</p>
  <div class="ol-hero-actions">
    <a class="ol-hero-cta" href="getting-started/">Try in 5 minutes →</a>
    <a class="ol-hero-cta ol-hero-cta--secondary" href="guides/python/">Python quickstart</a>
  </div>
  <p style="margin-top:1rem;margin-bottom:0"><a href="guides/start-here/">Not sure? Start here</a></p>
</div>

--8<-- "snippets/channel-banner.md"

Pick the path that matches how you work:

<div class="grid cards" markdown>

-   :material-language-rust: **Rust (crates.io)**

    ---

    Download `family.owl`, add three crates, run RDFS materialization—no clone required.

    [:octicons-arrow-right-24: Five-minute guide](getting-started/index.md#cratesio-only-no-clone)

-   :material-language-python: **Python**

    ---

    `pip install ontologos` — load OWL, classify with `profile="auto"`, incremental mutations.

    [:octicons-arrow-right-24: Python guide](guides/python.md)

-   :material-console: **CLI**

    ---

    `classify`, `materialize`, `explain` — build from this repository (`ontologos-cli` is not on crates.io).

    [:octicons-arrow-right-24: CLI reference](reference/cli.md)

-   :material-magnify: **Evaluate**

    ---

    Compare vs ELK, reasonable, and HermiT fixtures; 30-minute Pizza/Family playbook.

    [:octicons-arrow-right-24: Evaluator playbook](guides/evaluator-playbook.md)

</div>

!!! tip "No clone required for most users"
    Use **crates.io** or **PyPI**. Clone only to contribute, run benchmarks, or build the CLI.

<div class="ol-callout" markdown="0">
  <strong>Rust 1.88+</strong> for library users — see the <a href="guides/prerequisites.html">Prerequisites decision table</a>.
</div>

!!! warning "Mapped axiom counts ≠ Protégé totals"
    `axiom_count()` reflects **mapper output**, not every axiom Protégé displays. See [Supported constructs](reference/supported-constructs.md) and [Protégé axiom counts](guides/protege-axiom-counts.md).

!!! warning "Integration DO / DON'T"
    **DO** use CLI `ontologos classify`, Python `Reasoner(path=...).classify()`, or `ontologos_facade::classify` / profile crates.

    **DON'T** call classification on `ontologos_core::Reasoner` — use the [facade](guides/facade-api.md) instead. See [Choosing an API](guides/choosing-an-api.md).

!!! note "OWL DL / HermiT parity"
    On `main`, **`parity_pct`** and **`true_parity_pct`** are **100%** on the gated catalog (**889** in-scope cases; **450+428** runnable @ 30s). Published **v0.9.0** on PyPI/crates.io is EL/RL/RDFS-stable only. See [Evaluator scope](guides/evaluator-scope.md), [Profile stability](guides/profile-stability.md), and [Release status](project/release-status.md).

## What you need

| Channel | Link |
|---------|------|
| **Docs** | [ontologos.readthedocs.io](https://ontologos.readthedocs.io/en/latest/) |
| **crates.io** | [ontologos-core](https://crates.io/crates/ontologos-core) and siblings |
| **PyPI** | [`pip install ontologos`](https://pypi.org/project/ontologos/) |
| **Rust API** | [docs.rs/ontologos-core](https://docs.rs/ontologos-core/0.9.0) |
| **Changelog** | [project/changelog](project/changelog.md) · [GitHub](https://github.com/eddiethedean/ontologos/blob/main/CHANGELOG.md) |

## Documentation map {#documentation-map}

Use the sidebar for the full tree. Highlights:

| Area | Start here |
|------|------------|
| New users | [Start here](guides/start-here.md) · [Prerequisites](guides/prerequisites.md) |
| Examples | [Examples gallery](examples/index.md) |
| API choice | [Choosing an API](guides/choosing-an-api.md) · [Profile stability](guides/profile-stability.md) |
| Evaluate | [Evaluator playbook](guides/evaluator-playbook.md) · [Evaluator scope](guides/evaluator-scope.md) · [Comparison](comparison.md) |
| Upgrade | [Migration hub](migration/index.md) · [Release status](project/release-status.md) |

## Learning path

1. [Getting started — crates.io quick start](getting-started/index.md#cratesio-only-no-clone)
2. [Classify in five minutes](getting-started/classify-quickstart.md)
3. [Load an OWL file](getting-started/load-owl-file.md)
4. [RDFS materialization](getting-started/rdfs-materialization.md)
5. [OWL RL saturation](getting-started/owl-rl-saturation.md)
6. [OWL EL classification](getting-started/owl-el-classification.md)
7. [First ontology](getting-started/first-ontology.md) *(clone optional)*
8. [Profile detection](guides/profile-detection.md)
9. [JSON snapshots](json-snapshot-v2.md)
10. [Error reference](reference/errors.md)

## Capability matrix (published v0.9.0)

| Capability | Library | CLI | Python |
|------------|---------|-----|--------|
| Load OWL files | Yes | Yes | Yes |
| Profile detection | Yes | `profile` | `"auto"` |
| RDFS materialization | Yes | `materialize` | `profile="rdfs"` |
| OWL RL saturation | Yes | `classify --profile rl` | `profile="rl"` |
| OWL EL taxonomy | Yes | `classify --profile el` | `profile="el"` |
| OWL DL (pre-release) | Yes | `classify --profile dl` | `profile="dl"` |
| Incremental reasoning | Yes | `--incremental` | `incremental=True` |
| Explanations | Yes | `explain` | `explain()` |
| Taxonomy DataFrame export | No | No | Yes (pandas/polars) |
