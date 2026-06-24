# OntoLogos documentation

<div class="ol-hero" markdown="0">
  <div class="ol-hero-badges">
    <span class="ol-badge ol-badge--accent">v1.0.0 workspace</span>
    <span class="ol-badge">Rust 1.88+</span>
    <span class="ol-badge">OWL EL · RL · RDFS · DL preview</span>
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

    **DON'T** call `ontologos_core::Reasoner::classify()` directly — it is a stub. See [Choosing an API](guides/choosing-an-api.md).

!!! note "OWL DL / HermiT parity"
    `--profile dl` and preview modes are available for early testing. Full HermiT replacement is **in progress** (~58% in-scope catalog parity). Not suitable for production DL workflows yet. See [Preview profiles](guides/preview-profiles.md) and [Release status](project/release-status.md).

## What you need

| Channel | Link |
|---------|------|
| **Docs** | [ontologos.readthedocs.io](https://ontologos.readthedocs.io/en/latest/) |
| **crates.io** | [ontologos-core](https://crates.io/crates/ontologos-core) and siblings |
| **PyPI** | [`pip install ontologos`](https://pypi.org/project/ontologos/) |
| **Rust API** | [docs.rs/ontologos-core](https://docs.rs/ontologos-core/1.0.0) |
| **Changelog** | [project/changelog](project/changelog.md) · [GitHub](https://github.com/eddiethedean/ontologos/blob/main/CHANGELOG.md) |

## Documentation map {#documentation-map}

### Getting started

| Document | Description |
|----------|-------------|
| [Start here](guides/start-here.md) | Pick your path (single next step) |
| [Prerequisites](guides/prerequisites.md) | Rust, Python, clone vs crates.io |
| [Getting started overview](getting-started/index.md) | Success paths by persona |
| [First ontology](getting-started/first-ontology.md) | `OntologyBuilder` walkthrough |
| [Load an OWL file](getting-started/load-owl-file.md) | Parser, formats, `ParseMeta` |
| [RDFS materialization](getting-started/rdfs-materialization.md) | TBox closure |
| [OWL RL saturation](getting-started/owl-rl-saturation.md) | Forward-chaining |
| [OWL EL classification](getting-started/owl-el-classification.md) | Completion taxonomy |

### Guides

| Document | Description |
|----------|-------------|
| [Choosing an API](guides/choosing-an-api.md) | Crate and entry-point picker |
| [Facade API](guides/facade-api.md) | Unified `classify` routing |
| [Python](guides/python.md) | PyPI bindings |
| [Preview profiles](guides/preview-profiles.md) | DL / ALC preview status |
| [Evaluator playbook](guides/evaluator-playbook.md) | 30-minute evaluation |
| [Profile detection](guides/profile-detection.md) | EL / RL / QL / DL |
| [Glossary](guides/glossary.md) | Terminology |
| [Incremental reasoning](guides/incremental-reasoning.md) | Session API |
| [Production integration](guides/production-integration.md) | Services, untrusted input |
| [Troubleshooting](guides/troubleshooting.md) | Common fixes |
| [Comparison](comparison.md) | vs ELK, Konclude, reasonable |
| [Security](security.md) | Untrusted JSON and OWL |

### Reference

| Document | Description |
|----------|-------------|
| [Architecture](architecture.md) | Crate graph |
| [CLI](reference/cli.md) | Command-line tool |
| [Errors](reference/errors.md) | Error enums |
| [Supported constructs](reference/supported-constructs.md) | Mapped vs skipped OWL |
| [Explain](reference/explain.md) | Proof graphs |
| [Query](reference/query.md) | Taxonomy queries |
| [Conformance](reference/conformance.md) | HermiT-ported tests |
| [Rust API (docs.rs)](https://docs.rs/ontologos-core/1.0.0) | Generated API reference |

### Migration & project

| Document | Description |
|----------|-------------|
| [Upgrade to latest](migration/index.md) | v1.0.0 migration hub |
| [FAQ](project/faq.md) | Common questions |
| [Release status](project/release-status.md) | Channels and stability |
| [Contributing](project/contributing.md) | Contributor workflow |
| [Roadmap summary](project/roadmap-summary.md) | What ships next |

## Learning path

1. [First ontology](getting-started/first-ontology.md)
2. [Load an OWL file](getting-started/load-owl-file.md)
3. [RDFS materialization](getting-started/rdfs-materialization.md)
4. [OWL RL saturation](getting-started/owl-rl-saturation.md)
5. [OWL EL classification](getting-started/owl-el-classification.md)
6. [Profile detection](guides/profile-detection.md)
7. [JSON snapshots](json-snapshot-v2.md)
8. [Error reference](reference/errors.md)

## Capability matrix (v1.0.0)

| Capability | Library | CLI | Python |
|------------|---------|-----|--------|
| Load OWL files | Yes | Yes | Yes |
| Profile detection | Yes | `profile` | `"auto"` |
| RDFS materialization | Yes | `materialize` | `profile="rdfs"` |
| OWL RL saturation | Yes | `classify --profile rl` | `profile="rl"` |
| OWL EL taxonomy | Yes | `classify --profile el` | `profile="el"` |
| OWL DL (preview) | Yes | `classify --profile dl` | `profile="dl"` |
| Incremental reasoning | Yes | `--incremental` | `incremental=True` |
| Explanations | Yes | `explain` | `explain()` |
| Taxonomy DataFrame export | No | No | Yes (pandas/polars) |
