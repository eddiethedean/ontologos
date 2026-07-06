# Known limitations

Read this before integrating OntoLogos in production. These are **by design**, not bugs.

Canonical profile and channel guidance: [Profile stability matrix](profile-stability.md) · [Install and channels](install-channels.md).

## `owl:imports` behavior is format-dependent

Import handling depends on serialization format. See the canonical [OWL imports reference](../reference/owl-imports.md).

| Format | `load_ontology` / `load_ontology_in` | `ParseLimits::default()` |
|--------|--------------------------------------|--------------------------|
| **RDF/XML** (`.owl`, `.rdf`, `.xml`) | Local `owl:imports` **merged** (`merge_imports: true`) | **`merge_imports: false`** — set explicitly for custom loaders |
| **Turtle**, **OWL Functional** | **Not merged** — only axioms in the loaded file | Same |
| **Remote import IRIs** | **Never fetched** over the network | Same |

For untrusted uploads, use `load_ontology_in` with `ParseLimits { merge_imports: false, .. }` unless sibling import files are trusted. For multi-file ontologies with remote imports, merge upstream (e.g. [ROBOT](http://robot.obolibrary.org/)) before loading.

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

After reasoning, persist results with `Ontology::to_json()` (format v3 on published **1.0.0**; v2 still readable). There is no built-in OWL/RDF serializer — retain the source OWL file plus processing metadata.

## Rust integration

Use `ontologos_facade::classify` — not `Reasoner::classify()` on core. See [Rust integration contract](rust-integration-contract.md).

## Profile availability

Production profiles on **PyPI / crates.io** (published **1.0.0**): `rdfs`, `rl`, `el`, `auto`, `dl`, and `swrl`. Preview: `alc`, `dl-preview`.

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
