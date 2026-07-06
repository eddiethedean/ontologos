# Start here

**Before you start:** Read [Before you integrate](before-you-integrate.md). See [Prerequisites](prerequisites.md) for Rust version and tooling. Unfamiliar with OWL terms? See the [Glossary](glossary.md) and [When not to use OntoLogos](when-not-to-use.md). You do **not** need to clone this repository to try OntoLogos — `pip install ontologos` or crates.io dependencies are enough for most workflows. Install channels: [Install and channels](install-channels.md).

## New to OWL?

1. [Glossary](glossary.md) — classes, profiles, subsumption
2. [When not to use OntoLogos](when-not-to-use.md) — honest fit check
3. [Python guide](python.md) or [Rust quickstart](../getting-started/index.md#cratesio-only-no-clone) — five-minute try

## Bindings (Node, Java, .NET, C, WASM)

Source-build from a clone — see [Bindings overview](bindings-overview.md). Rust and Python are on crates.io/PyPI at **v1.1.1**.

Pick the path that matches how you work. Each link is a single next step—not the full documentation map.

## Try it in five minutes (no clone)

[Getting started — crates.io only](../getting-started/index.md#cratesio-only-no-clone) — download `family.owl`, add `ontologos-core`, `ontologos-parser`, and `ontologos-rl`, run RDFS materialization.

**Python instead:** `pip install ontologos` → [Python guide](python.md).

## I want to load an OWL file and classify it

1. [Classify quick start](../getting-started/classify-quickstart.md) — `ontologos-facade::classify` in five minutes (no clone)
2. [Load an OWL file](../getting-started/load-owl-file.md) — formats, `ParseMeta`, imports limitation
3. [Choosing an API](choosing-an-api.md) — which crate and entry point
4. [Profile stability matrix](profile-stability.md) — production vs pre-release profiles

CLI shortcut (not on crates.io):

See [CLI installation](../getting-started/cli-install.md) — then:

```bash
ontologos classify --profile auto family.owl
```

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

## I need SWRL rules

[SWRL quick start](../getting-started/swrl.md) — DLSafe rules + DL on v1.0.0.

## I am evaluating vs HermiT / ELK / reasonable

[Evaluator scope](evaluator-scope.md) · [Evaluator playbook](evaluator-playbook.md) · [When not to use OntoLogos](when-not-to-use.md) · [Comparison](../comparison.md)

**Evaluate with Python only (no Rust, no clone):**

```bash
pip install ontologos==1.1.1
curl -L -o family.owl \
  https://raw.githubusercontent.com/eddiethedean/ontologos/main/benchmarks/data/family.owl
python -c "from ontologos import Reasoner; r=Reasoner(path='family.owl',profile='rl').classify(); print(r)"
```

**Production OWL DL:** Stable on **PyPI/crates.io 1.0.0** (`profile="dl"`). HermiT catalog parity applies to **889 gated in-scope cases** — [validate on your corpus](evaluator-scope.md).

## I am integrating in a service

[Production integration](production-integration.md) · [Deployment](deployment.md) · [Security](../security.md) · [Rust integration contract](rust-integration-contract.md)

## I want to contribute

[Contributing](../project/contributing.md) on this site · [GitHub CONTRIBUTING](https://github.com/eddiethedean/ontologos/blob/main/CONTRIBUTING.md) · [Architecture](../architecture.md)

## I am upgrading an existing integration

[Upgrade to latest](../migration/index.md) — **v1.1.1** published on crates.io and PyPI; see [v1.0.x → v1.1.0](../migration/v1.0.x-to-v1.1.0.md) if upgrading from 1.0.

## Common questions

[FAQ](../project/faq.md) · [Troubleshooting](troubleshooting.md) · [Protégé axiom counts](protege-axiom-counts.md)

## Full documentation map

Return to the [documentation home](../index.md#documentation-map) for the complete table of contents.
