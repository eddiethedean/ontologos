# Before you integrate

Read this **once** before loading ontologies or calling `classify()` in production.

## Partial OWL mapping

The parser maps a **subset** of OWL constructs into the core model. Complex class expressions, many data-property axioms, and some property axiom shapes are scanned for profile detection but **skipped** during mapping.

- `ontology.axiom_count()` is **mapper output**, not Protégé's logical axiom total.
- **Family.owl:** ~57 mapped axioms is normal.
- Skipped items appear in `parse_meta.skipped_axiom_count` and `parse_meta.warnings`.

See [Protégé axiom counts](protege-axiom-counts.md) · [Supported constructs](../reference/supported-constructs.md).

## `owl:imports`

| Format | Local imports merged? | Remote URLs fetched? |
|--------|----------------------|---------------------|
| RDF/XML | Yes (default) | **Never** |
| Turtle, OWL Functional | No | **Never** |

Merge multi-file bundles with [ROBOT](http://robot.obolibrary.org/) before loading when imports are remote or cross-format.

See [OWL imports reference](../reference/owl-imports.md) · [Known limitations](known-limitations.md).

## Rust API footgun

Use `ontologos_parser::load_ontology` and **`ontologos_facade::classify(&mut reasoner)`** — not `Ontology::from_file` or `reasoner.classify()` on core.

See [Rust integration contract](rust-integration-contract.md).

## OWL DL scope

HermiT parity metrics apply to **889 gated conformance cases**, not every real-world ontology. Validate DL results on **your** corpus before production.

See [Evaluator scope](evaluator-scope.md) · [When not to use OntoLogos](when-not-to-use.md).

## Release channel

**Published today:** v1.1.3 on crates.io and PyPI. Pin install commands to [Release status](../project/release-status.md).

## Next steps

| Goal | Guide |
|------|-------|
| Quick try (Python) | [Python guide](python.md) |
| Quick try (Rust) | [Getting started — crates.io](../getting-started/index.md#cratesio-only-no-clone) |
| Pick a crate | [Choosing an API](choosing-an-api.md) |
| Troubleshooting | [Troubleshooting](troubleshooting.md) |
