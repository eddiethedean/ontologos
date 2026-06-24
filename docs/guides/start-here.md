# Start here

**Before you start:** See [Prerequisites](prerequisites.md) for Rust version and tooling. You do **not** need to clone this repository to try OntoLogos — `pip install ontologos` or crates.io dependencies are enough for most workflows.

Pick the path that matches how you work. Each link is a single next step—not the full documentation map.

## Try it in five minutes (no clone)

[Getting started — crates.io only](../getting-started/index.md#cratesio-only-no-clone) — download `family.owl`, add three crates to `Cargo.toml`, run RDFS materialization.

**Python instead:** `pip install ontologos` → [Python guide](python.md).

## I want to load an OWL file and classify it

1. [Load an OWL file](../getting-started/load-owl-file.md) — formats, `ParseMeta`, axiom counts
2. [Choosing an API](choosing-an-api.md) — which crate and entry point
3. [Profile detection](profile-detection.md) — EL / RL / RDFS / DL routing

CLI shortcut (from a clone): `ontologos classify --profile auto ontology.owl`

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

**Production OWL DL today:** use Protégé + HermiT or Konclude. OntoLogos DL is improving but not a drop-in HermiT replacement yet — see [Preview profiles](preview-profiles.md).

## I am integrating in a service

[Production integration](production-integration.md) · [Security](../security.md) · [Choosing an API](choosing-an-api.md)

!!! warning "Do not call the core stub directly"
    Use `ontologos_facade::classify`, profile crates (`ElClassifier`, `RlEngine`, `RdfsEngine`), CLI, or Python `Reasoner` — not `ontologos_core::Reasoner::classify()` (returns `NotImplemented`).

## I want to contribute

[Contributing](../project/contributing.md) on this site · [GitHub CONTRIBUTING](https://github.com/eddiethedean/ontologos/blob/main/CONTRIBUTING.md) · [Architecture](../architecture.md)

## I am upgrading an existing integration

[Upgrade to latest](../migration/index.md) — jump to v1.0.0 from any prior release.

## Common questions

[FAQ](../project/faq.md) · [Troubleshooting](troubleshooting.md) · [Protégé axiom counts](protege-axiom-counts.md)

## Full documentation map

Return to the [documentation home](../index.md#documentation-map) for the complete table of contents.
