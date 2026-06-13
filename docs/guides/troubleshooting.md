# Troubleshooting

## `cargo test` fails on Pizza / Family tests

**Symptom:** `missing benchmark corpus pizza at ...`

**Fix:** Download corpora before testing:

```bash
./benchmarks/scripts/download.sh
cargo test --workspace
```

CI runs `download.sh` automatically. See [benchmarks](../project/benchmarks.md).

## `ontologos profile` fails: file not found

Pizza is not committed to the repo (gitignored). Run `./benchmarks/scripts/download.sh` or use `benchmarks/data/family.owl` (vendored).

## Axiom count does not match Protégé

See the dedicated walkthrough: [Protégé vs OntoLogos axiom counts](protege-axiom-counts.md).

`ontology.axiom_count()` is the number of axioms **stored in core** after mapping, not the raw OWL logical axiom count in the source file.

The parser skips complex class expressions, many data-property axioms, and some property axiom shapes. Named ABox axioms (`ClassAssertion`, `ObjectPropertyAssertion`, `SameIndividual`, `DifferentIndividuals`) are mapped in v0.4. Skipped items appear in `parse_meta.skipped_axiom_count` and `parse_meta.warnings`.

Benchmark manifest values (e.g. Pizza `658`) are mapper output targets, documented in [benchmarks](../project/benchmarks.md).

## Pizza is DL with mapped-construct diagnostics

Expected for the current Pizza corpus: mapped axioms include both EL shapes (existentials) and constructs outside EL (e.g. inverse properties), so detection reports **DL** with diagnostics explaining which mapped constructs rule out EL/RL. Source-only constructs may also appear under the [hybrid profile contract](profile-detection.md).

## `Ontology::from_file` returns `ParseNotAvailable`

Use `ontologos_parser::load_ontology`. See [FAQ](../project/faq.md).

## JSON `from_json` fails

Common causes: `format_version: 1`, invalid IRI, unknown entity in axiom, size limit. See [errors.md](../reference/errors.md) and [json-snapshot-v2.md](../json-snapshot-v2.md).

## Unsupported file extension

`load_ontology` returns `Error::UnsupportedFormat` for unrecognized extensions. Supported: `.owl`, `.rdf`, `.xml`, `.ttl`, `.turtle`, `.ofn`, `.func`.

## Parser warnings after load

Inspect `ontology.parse_meta().warnings`. Warnings are non-fatal; the ontology loads with whatever axioms could be mapped.

## Missing axioms from imported ontologies

`owl:imports` is recorded in `parse_meta` but **not resolved** — only axioms in the loaded file are mapped. Merge imports upstream or use a single bundle file.

## `classify` / `explain` behavior

CLI **`classify --profile auto|el|rl|rdfs`** routes to EL taxonomy, RL saturation, or RDFS materialization. Use **`materialize`** for explicit RDFS. **`explain`** is available in v0.9.0.

Library users: call `ontologos_el::classify_with_profile`, `ontologos_rdfs::classify_reasoner`, or `ontologos_rl::classify_reasoner`. **Do not** call `ontologos_core::Reasoner::classify()` directly — it returns delegate hints for RDFS/RL and `NotImplemented` for EL/Auto. CLI and Python route correctly. See [CLI reference](../reference/cli.md), [errors.md](../reference/errors.md), and [Choosing an API](../guides/choosing-an-api.md).

## RDFS does not expand `EquivalentClasses`

RDFS materializes primarily `subClassOf` transitive closure via reasonable. Some RDFS rules (`subPropertyOf` transitivity, domain/range inheritance) have [upstream gaps](../reference/reasonable-limits.md). Named `EquivalentClasses` axioms are expanded into mutual subsumption by `ontologos-rl` during saturation. ABox `rdf:type` propagation is handled by RL rules when using `ontologos-rl` or Python `profile="rl"`.
