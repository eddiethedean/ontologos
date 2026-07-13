# Deployment and observability

Patterns for running OntoLogos in containers and production services. Security: [Security](../security.md). Integration: [Production integration](production-integration.md).

## Container deployment

### Minimal Python sidecar

```dockerfile
FROM python:3.12-slim
RUN pip install --no-cache-dir ontologos==1.1.3
WORKDIR /data
COPY reason.py .
ENTRYPOINT ["python", "reason.py"]
```

Mount ontologies read-only; never bake secrets into images.

### Rust service binary

Multi-stage build:

```dockerfile
FROM rust:1.88-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p my-reasoner-service

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/my-reasoner-service /usr/local/bin/
ENTRYPOINT ["my-reasoner-service"]
```

Pin `rust:1.88` or newer — see [Prerequisites](prerequisites.md).

### Resource sizing (indicative)

| Workload | Memory | CPU | Notes |
|----------|--------|-----|-------|
| Family RL | < 256 MiB | 1 core | Smoke / dev |
| Pizza EL | ~512 MiB–1 GiB | 1–2 cores | ~658 mapped axioms |
| OWL DL @ 30s budget | 1–4 GiB | 2–4 cores | Set `budget_secs`; tableau is CPU-bound |
| Untrusted uploads | Cap via `ParseLimits` / `Limits` | — | See [Security](../security.md) |

DL memory scales with ontology size and tableau expansion. Load-test your corpus before fixing limits in Kubernetes.

## DL wall-clock budgets

```rust
use ontologos_core::{ReasonerConfig, Reasoner};

let config = ReasonerConfig {
    budget_secs: Some(30),
    parallelism: 4,
    ..ReasonerConfig::default()
};
```

Environment fallback: `ONTOLOGOS_DL_BUDGET_SECS=30`.

**Production rule:** always check `ConsistencyResult::complete` before acting on `consistent`.

## Observability

### Consistency and classify JSON

`ConsistencyResult` implements `Serialize` — log for audit trails:

```rust
let result = ontologos_facade::check_consistency(&reasoner)?;
let json = serde_json::to_string(&result)?;
tracing::info!(%json, "consistency_check");
```

Facade helpers: `taxonomy_json`, `rl_materialization_json`, `rdfs_materialization_json`.

### DL performance timings

When `ONTOLOGOS_DL_PERF=1`, `ontologos-dl` records `DlPerfTimings` (saturation vs tableau phases). Use for profiling, not default production logging.

### Parse metadata

After file load, log `ontology.parse_meta()` summary:

- `mapped_axiom_count`, `skipped_axiom_count`
- `warnings` (non-fatal mapping issues)
- `profile_constructs` (detection input)

Python: `reasoner.parse_meta` dict after construction from file.

### Tracing

Workspace crates use the `tracing` crate. Enable in your binary:

```rust
tracing_subscriber::fmt::init();
```

Set `RUST_LOG=ontologos_dl=debug,ontologos_el=info` for engine-level spans (crate names use underscores in filter directives).

### Health checks

Recommended liveness probe:

1. Load a small vendored ontology (`family.owl`)
2. Run `classify` with known profile (`rl`)
3. Assert `inferred_total() > 0` or expected subsumption count

Avoid DL in liveness probes — use RL or EL for predictable latency.

## Horizontal scaling

OntoLogos reasoners are **in-process** — scale by replicating stateless worker pods behind a queue. Each job should:

1. Load ontology (or snapshot JSON)
2. Classify / materialize
3. Persist results (`to_json()` or application DB)
4. Drop in-memory state

Incremental sessions are **single-process** — do not share `Reasoner` across threads without external synchronization.

## Related

- [Performance and scaling](performance.md)
- [Production integration](production-integration.md)
- [Troubleshooting](troubleshooting.md)
