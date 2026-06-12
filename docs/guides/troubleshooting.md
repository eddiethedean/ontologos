# Troubleshooting

## `cargo test` fails on Pizza / Family tests

**Symptom:** `missing benchmark corpus pizza at ...`

**Fix:** Download corpora before testing:

```bash
./benchmarks/scripts/download.sh
cargo test --workspace
```

CI runs `download.sh` automatically. See [benchmarks/README.md](../../benchmarks/README.md).

## `ontologos profile` fails: file not found

Pizza is not committed to the repo (gitignored). Run `./benchmarks/scripts/download.sh` or use `benchmarks/data/family.owl` (vendored).

## Axiom count does not match Protégé

`ontology.axiom_count()` is the number of axioms **stored in core** after mapping, not the raw OWL logical axiom count in the source file.

The parser skips complex class expressions, most ABox axioms, and many property axiom shapes. Skipped items appear in `parse_meta.skipped_axiom_count` and `parse_meta.warnings`.

Benchmark manifest values (e.g. Pizza `658`) are mapper output targets, documented in [benchmarks/README.md](../../benchmarks/README.md).

## Pizza is DL with mapped-construct diagnostics

Expected for the current Pizza corpus: mapped axioms include both EL shapes (existentials) and constructs outside EL (e.g. inverse properties), so detection reports **DL** with diagnostics explaining which mapped constructs rule out EL/RL. Source-only constructs may also appear under the [hybrid profile contract](profile-detection.md).

## `Ontology::from_file` returns `ParseNotAvailable`

Use `ontologos_parser::load_ontology`. See [FAQ](../../FAQ.md).

## JSON `from_json` fails

Common causes: `format_version: 1`, invalid IRI, unknown entity in axiom, size limit. See [errors.md](../reference/errors.md) and [json-snapshot-v2.md](../json-snapshot-v2.md).

## Unsupported file extension

`load_ontology` returns `Error::UnsupportedFormat` for unrecognized extensions. Supported: `.owl`, `.rdf`, `.xml`, `.ttl`, `.turtle`, `.ofn`, `.func`.

## Parser warnings after load

Inspect `ontology.parse_meta().warnings`. Warnings are non-fatal; the ontology loads with whatever axioms could be mapped.

## Missing axioms from imported ontologies

`owl:imports` is recorded in `parse_meta` but **not resolved** — only axioms in the loaded file are mapped. Merge imports upstream or use a single bundle file.

## `classify` / `explain` behavior

In v0.3, **`classify`** and **`materialize`** both run RDFS TBox materialization and emit the same inference report (`status` differs). OWL EL/RL classification ships in v0.5; **`explain`** in v0.6.

Library users: call `ontologos_rdfs::classify_reasoner` for `Profile::Rdfs`. `Reasoner::classify()` returns a delegate hint for `Profile::Rdfs` and `NotImplemented` for `Profile::Auto` / `El` / `Rl`. See [CLI reference](../reference/cli.md) and [errors.md](../reference/errors.md).

## RDFS does not expand `EquivalentClasses`

v0.3 RDFS materializes `subClassOf` / `subPropertyOf` closure and object-property domain/range inheritance only. Named `EquivalentClasses` axioms are stored but not expanded into mutual subsumption (planned for RL/EL engines). `rdf:type` propagation requires ABox support (v1.6).
