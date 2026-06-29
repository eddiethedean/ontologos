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

Profile **classification** uses mapped TBox shapes (`parse_meta.profile_constructs`). **Diagnostics** explain why EL/RL/QL do not apply (mapped violations) and may also report constructs seen in the full parse (`parse_meta.constructs`) that were not mapped into core. Pizza is detected as **DL** because mapped axioms mix EL and RL-forbidden shapes.

Manifest `axiom_count_approx` values (e.g. Pizza `658`) are **mapper output** counts (`ontology.axiom_count()` / `mapped_axiom_count`), not raw OWL logical axiom totals from the source file.

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

## EL benchmarks (v0.5)

| Script | Purpose |
|--------|---------|
| `scripts/compare-pizza-el-golden.sh` | Pizza EL golden regression gate (in-house EL vs committed JSON) |
| `scripts/compare-classification-fixtures.sh` | HermiT Tier B gate — pizza/wine/galen/propreo XML vs HermiT hierarchy goldens |
| `scripts/compare-tier-c-gate.sh` | Tier C PR gate — `family.owl` DL golden + Pizza EL golden |
| `scripts/compare-dl-hermit-crosscheck.sh` | Optional/nightly HermiT JAR ⊆ OntoLogos DL cross-check |
| `scripts/download-hermit-jar.sh` | Fetch standalone HermiT CLI JAR to `benchmarks/data/hermit.jar` |
| `scripts/benchmark-dl-perf.sh` | DL classification wall-time snapshot |
| `scripts/generate-go-subset.sh` | Trim GO with ROBOT for `<10s` EL CI test |

EL integration tests: `cargo test -p ontologos-el --test pizza_el`

## HermiT conformance burndown

OntoLogos v1.0 reached **100% catalog parity** (`parity_pct = 100%`, **889** in-scope cases) with the full gated suite green @ 30s on `main`. Contributors should read **[docs/guides/hermit-burndown.md](../docs/guides/hermit-burndown.md)** before touching `ontologos-conformance` or `ontologos-dl`.

```bash
bash benchmarks/scripts/hermit-burndown.sh status   # parity dashboard
bash benchmarks/scripts/hermit-burndown.sh loop     # daily workflow
```

HermiT fixtures live under `benchmarks/data/hermit/`. Catalog tooling: `tests/hermit/`.
