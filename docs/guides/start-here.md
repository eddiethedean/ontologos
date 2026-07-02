# Start here

**Before you start:** See [Prerequisites](prerequisites.md) for Rust version and tooling. Unfamiliar with OWL terms? See the [Glossary](glossary.md). You do **not** need to clone this repository to try OntoLogos — `pip install ontologos` or crates.io dependencies are enough for most workflows. Install channels: [Install and channels](install-channels.md).

Pick the path that matches how you work. Each link is a single next step—not the full documentation map.

## Try it in five minutes (no clone)

[Getting started — crates.io only](../getting-started/index.md#cratesio-only-no-clone) — download `family.owl`, add `ontologos-core`, `ontologos-parser`, and `ontologos-rl`, run RDFS materialization.

**Python instead:** `pip install ontologos` → [Python guide](python.md).

## I want to load an OWL file and classify it

1. [Classify quick start](../getting-started/classify-quickstart.md) — `ontologos-facade::classify` in five minutes (no clone)
2. [Load an OWL file](../getting-started/load-owl-file.md) — formats, `ParseMeta`, imports limitation
3. [Choosing an API](choosing-an-api.md) — which crate and entry point
4. [Profile stability matrix](profile-stability.md) — production vs pre-release profiles

CLI shortcut (requires git install — not on crates.io):

```bash
cargo install --git https://github.com/eddiethedean/ontologos ontologos-cli
ontologos classify --profile auto family.owl
```

Or build from a clone. See [CLI reference](../reference/cli.md) and [Install and channels](install-channels.md).

## I am building ontologies in Rust

[First ontology](../getting-started/first-ontology.md) — `OntologyBuilder`, subclass axioms, taxonomy queries.

## I need RDFS or OWL RL only

| Goal | Guide |
|------|-------|
| RDFS TBox materialization | [RDFS materialization](../getting-started/rdfs-materialization.md) |
| OWL RL forward-chaining | [OWL RL saturation](../getting-started/owl-rl-saturation.md) |

Prefer CLI **`materialize`** for explicit RDFS (same engine as `classify --profile rdfs`).

## I need OWL EL taxonomy

[OWL EL classification](../getting-started/owl-el-classification.md) — in-house completion engine in `ontologos-el`.

## I am evaluating vs HermiT / ELK / reasonable

[Evaluator playbook](evaluator-playbook.md) · [Comparison](../comparison.md) · [Conformance coverage](../reference/conformance.md)

**Production OWL DL:** Not on PyPI **0.9.0**. On **`main` / workspace 1.0.0**, DL is gated for HermiT catalog parity — use the [Evaluator playbook](evaluator-playbook.md) on your corpus before production. For DL-only workflows today, HermiT/Konclude remain the default comparison baseline. See [Profile stability matrix](profile-stability.md).

## I am integrating in a service

[Production integration](production-integration.md) · [Security](../security.md) · [Choosing an API](choosing-an-api.md)

!!! warning "Do not call the core stub directly"
    Use `ontologos_facade::classify`, profile crates (`ElClassifier`, `RlEngine`, `ontologos_rl::rdfs::RdfsEngine`), CLI, or Python `Reasoner` — not `ontologos_core::Reasoner::classify()` (deprecated since 1.0.0).

## I want to contribute

[Contributing](../project/contributing.md) on this site · [GitHub CONTRIBUTING](https://github.com/eddiethedean/ontologos/blob/main/CONTRIBUTING.md) · [Architecture](../architecture.md)

## I am upgrading an existing integration

[Upgrade to latest](../migration/index.md) — published **v0.9.0** and upcoming **v1.0.0** paths.

## Common questions

[FAQ](../project/faq.md) · [Troubleshooting](troubleshooting.md) · [Protégé axiom counts](protege-axiom-counts.md)

## Full documentation map

Return to the [documentation home](../index.md#documentation-map) for the complete table of contents.
