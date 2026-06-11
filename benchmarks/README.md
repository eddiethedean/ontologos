# Benchmarks

Benchmark ontology corpora for integration and conformance testing.

| Corpus | Source | In repo? | CI |
|--------|--------|----------|-----|
| **Pizza** | [owlcs/pizza-ontology](https://github.com/owlcs/pizza-ontology) | Downloaded | Yes (`download.sh`) |
| **Family** | [rexster family.swrl.owl](https://github.com/martinhbramwell/Monetary-Ontology-Walkabout/blob/master/rexster/extension/example/src/main/resources/data/family.swrl.owl) | Vendored (`family.owl`) | Checksum verified |
| GALEN, GO, SNOMED | See manifest | Manual | Optional `#[ignore]` stress tests |

SHA-256 pins live in [`checksums.sha256`](checksums.sha256).

## Manifest

See [manifest.toml](manifest.toml) for the canonical list of ontologies, expected OWL profiles, source URLs, and licenses.

## Downloading OWL corpora

```bash
./benchmarks/scripts/download.sh
```

This fetches Pizza and verifies checksums. `family.owl` is committed; refresh from upstream with:

```bash
./benchmarks/scripts/download.sh --update-family
# then update benchmarks/checksums.sha256 if the file changed
```

GALEN, Gene Ontology, and SNOMED subsets require manual download (see manifest notes).

## Integration tests

| Test | When it runs |
|------|----------------|
| `mapping_fixtures.rs` | Always (synthetic minimal fixtures, all formats) |
| `manifest_integration.rs` | Always (Pizza + Family; requires `download.sh`) |
| `corpus_stress.rs` | `cargo test -- --ignored` when large files are present |

### Hybrid profile contract

Profile **classification** uses mapped TBox shapes (`parse_meta.profile_constructs`). **Diagnostics** also report constructs seen in the full parse (`parse_meta.constructs`) that fall outside the detected profile—for example Pizza is detected as **EL** but diagnostics mention DL constructs such as `ObjectAllValuesFrom` that were not mapped into core.

Manifest `axiom_count` values (e.g. Pizza `1056`) are **mapper output** counts (`ontology.axiom_count()` / `mapped_axiom_count`), not raw OWL logical axiom totals from the source file.

### Local testing

```bash
# Default CI-equivalent run
./benchmarks/scripts/download.sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# Parser integration only
cargo test -p ontologos-parser

# Optional stress (after manual download)
cargo test -p ontologos-parser --test corpus_stress -- --ignored
```

## Criterion benchmarks

```bash
cargo bench -p ontologos-core
```

Results are written under `target/criterion/`.
