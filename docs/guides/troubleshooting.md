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

Benchmark manifest values (e.g. Pizza `1056`) are mapper output targets, documented in [benchmarks/README.md](../../benchmarks/README.md).

## Pizza is EL but diagnostics mention DL constructs

Expected under the [hybrid profile contract](profile-detection.md). Detection uses mapped axioms; diagnostics flag constructs seen in the source outside the detected profile.

## `Ontology::from_file` returns `ParseNotAvailable`

Use `ontologos_parser::load_ontology`. See [FAQ](../../FAQ.md).

## JSON `from_json` fails

Common causes: `format_version: 1`, invalid IRI, unknown entity in axiom, size limit. See [errors.md](../reference/errors.md) and [json-snapshot-v2.md](../json-snapshot-v2.md).

## Unsupported file extension

`load_ontology` returns `Error::UnsupportedFormat` for unrecognized extensions. Supported: `.owl`, `.rdf`, `.xml`, `.ttl`, `.turtle`, `.ofn`, `.func`.

## Parser warnings after load

Inspect `ontology.parse_meta().warnings`. Warnings are non-fatal; the ontology loads with whatever axioms could be mapped.

## `classify` / `materialize` / `explain` not implemented

Reasoning engines ship in v0.3–v0.6 (`materialize` v0.3, `classify` v0.5, `explain` v0.6). Only `profile` is functional in v0.2. See [CLI reference](../reference/cli.md).
