# Known limitations

Read this before integrating OntoLogos in production. These are **by design**, not bugs.

Canonical profile and channel guidance: [Profile stability matrix](profile-stability.md) · [Install and channels](install-channels.md).

## `owl:imports` are not resolved

OntoLogos loads **one file** only. Imported ontologies are recorded in `parse_meta` but **not** fetched or merged. Axioms from imports are absent unless you merge files first.

**Workaround:** Bundle imports with [ROBOT](http://robot.obolibrary.org/) (`robot merge --input ontology.owl --output merged.owl`) or OWL API, then load the merged file.

See [Load an OWL file](../getting-started/load-owl-file.md).

## Partial OWL mapping

The parser maps a **subset** of OWL constructs into the core model. Complex class expressions, many data-property axioms, and some property axiom shapes are scanned for profile detection but **skipped** during mapping.

- `ontology.axiom_count()` is **mapper output**, not Protégé's logical axiom total.
- Skipped items appear in `parse_meta.skipped_axiom_count` and `parse_meta.warnings`.

**Family.owl (vendored):** ~57 mapped axioms is normal.

See [Protégé axiom counts](protege-axiom-counts.md) · [Supported constructs](../reference/supported-constructs.md).

## Not a Protégé replacement

OntoLogos is library-first orchestration for Rust/Python services. It does not provide interactive OWL editing, plugin ecosystems, or full desktop authoring workflows.

## No OWL export

After reasoning, persist results with `Ontology::to_json()` (format v3 on workspace 1.0.0; v2 still readable). There is no built-in OWL/RDF serializer — retain the source OWL file plus processing metadata.

## Rust: do not call `Reasoner::classify()` on core

Use `ontologos_facade::classify`, profile crates (`RdfsEngine`, `RlEngine`, `ElClassifier`), CLI, or Python `Reasoner`. Core `Reasoner::classify()` is a stub.

See [Facade API](facade-api.md) · [Rust API in 60 seconds](../getting-started/index.md#rust-api-in-60-seconds).

## Profile availability by install channel

| Profile | PyPI / crates.io 0.9.0 | `main` workspace 1.0.0 |
|---------|------------------------|-------------------------|
| `rdfs`, `rl`, `el`, `auto` | Production-supported | Production-supported |
| `dl` | Not production-supported | Stable (gated conformance) |
| `swrl` | Not available | Stable (DLSafe subset) |
| `alc`, `dl-preview` | Preview / errors | Preview |

Full matrix: [Profile stability](profile-stability.md).

## RDFS / reasonable upstream gaps

Some RDFS rules (`subPropertyOf` transitivity, domain/range inheritance) have [upstream gaps](../reference/reasonable-limits.md). Named `EquivalentClasses` expansion is handled by `ontologos-rl` during saturation.

## HermiT parity scope

`parity_pct = 100%` applies to the **gated conformance catalog** (889 in-scope cases), not every real-world ontology. Validate DL results on your corpus before production cutover.

See [Evaluator scope](evaluator-scope.md).

## Related

- [Troubleshooting](troubleshooting.md)
- [FAQ](../project/faq.md)
- [Security](../security.md)
