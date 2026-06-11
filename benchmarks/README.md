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
| `manifest_integration.rs` | Always (Pizza + Family) |
| `corpus_stress.rs` | `cargo test -- --ignored` when large files are present |

```bash
# Default CI-equivalent run
./benchmarks/scripts/download.sh
cargo test -p ontologos-parser

# Optional stress (after manual download)
cargo test -p ontologos-parser --test corpus_stress -- --ignored
```

## Criterion benchmarks (v0.1)

```bash
cargo bench -p ontologos-core
```

Results are written under `target/criterion/`.
