# Performance and Scaling

Guidance for sizing OntoLogos workloads. Optional incremental EL/RL/RDFS when `ReasonerConfig::incremental` is set. Benchmark numbers are indicative — run `cargo test -p ontologos-el --test incremental_bench -- --ignored` on your hardware.

## Default limits

### JSON deserialization

| Limit | Default | Configure via |
|-------|---------|---------------|
| `max_json_bytes` | 16 MiB | `Limits::max_json_bytes` |
| `max_entities` | 1,000,000 | `Limits::max_entities` |
| `max_axioms` | 10,000,000 | `Limits::max_axioms` |
| `max_iri_len` | 8,192 | `Limits::max_iri_len` |

See [Security](../security.md) and [JSON snapshot v3](../json-snapshot-v3.md).

### OWL file parsing

| Limit | Default | Configure via |
|-------|---------|---------------|
| `max_file_bytes` | 64 MiB | `ParseLimits::max_file_bytes` |
| `max_axioms` | 10,000,000 | `ParseLimits::max_axioms` |

See [`ParseLimits`](https://docs.rs/ontologos-parser/0.9.0/ontologos_parser/struct.ParseLimits.html).

## Engine behavior

### RDFS (`RdfsEngine`)

- **Complexity:** Depends on taxonomy depth and property hierarchy size; runs until TBox rules saturate.
- **Memory:** In-place — inferred axioms are added to the same `Ontology`.
- **Parallelism:** Sequential only.

### OWL RL (`RlEngine`)

- **Parallelism:** `RlEngine::new(n)` with `n` in `1..=64`. Parallelism affects ABox type-rule candidate expansion only; use `1` for fully sequential execution.
- **Pipeline:** Always runs RDFS materialization first, then RL rules to fixed point.

```rust
let report = RlEngine::try_new(4)?.saturate(&mut ontology)?;
```

For reproducible debugging, use `RlEngine::new(1)`.

## Reference corpora

| Corpus | Mapped axioms (approx.) | Profile | Notes |
|--------|-------------------------|---------|-------|
| Family | ~57 | RL | Vendored; good RL smoke test |
| Pizza | ~658 | DL | Requires `download.sh`; stress parser + profile |

Run locally:

```bash
./benchmarks/scripts/download.sh
cargo test -p ontologos-parser --test manifest_integration
cargo test -p ontologos-rl --test corpus
```

Optional Criterion bench:

```bash
cargo bench -p ontologos-core
```

Results under `target/criterion/`.

## External comparison (RL)

For OWL RL materialization baselines, compare against [reasonable](https://github.com/gtfierro/reasonable) using the optional harness:

```bash
./benchmarks/scripts/compare-reasonable.sh benchmarks/data/family.owl
```

See [Conformance coverage](../reference/conformance.md).

## Scaling recommendations

| Workload | Recommendation |
|----------|----------------|
| Small ontologies (< 10k mapped axioms) | Default limits; `RlEngine::new(1)` |
| Medium batch jobs | Tune `ParseLimits`; snapshot to JSON v3 after saturation |
| Untrusted uploads | `load_ontology_in(base, path)` + reduced limits — [Production integration](production-integration.md) |
| Large DL corpora | Set `ReasonerConfig::budget_secs` or `ONTOLOGOS_DL_BUDGET_SECS`; validate on your hardware |
| Incremental updates | `ReasonerConfig::incremental`; CLI `--incremental` |

## Related

- [Benchmarks (maintainers)](../project/benchmarks.md)
- [Security](../security.md)
- [Comparison](../comparison.md)
