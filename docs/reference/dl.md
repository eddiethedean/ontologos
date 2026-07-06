# OWL DL API Reference

OWL 2 DL classification and consistency via [`ontologos-dl`](https://docs.rs/ontologos-dl/1.1.1), routed through [`ontologos-facade`](facade.md).

!!! note "Install channel"
    **PyPI / crates.io 1.0.0:** `Profile::Dl` is production-supported. Validate on your corpus — see [Evaluator scope](../guides/evaluator-scope.md) and [Install channels](../guides/install-channels.md).

Narrative: [Production integration — OWL DL](../guides/production-integration.md#owl-dl-in-production) · [Evaluator scope](../guides/evaluator-scope.md).

## Entry points

Prefer the facade for CLI/Python parity:

```rust
use ontologos_core::{Profile, Reasoner, ReasonerConfig};
use ontologos_facade::{check_consistency, classify, ClassifyOutcome};
use ontologos_parser::load_ontology;

let mut reasoner = Reasoner::builder()
    .profile(Profile::Dl)
    .config(ReasonerConfig {
        budget_secs: Some(30),
        ..ReasonerConfig::default()
    })
    .build(load_ontology(path)?)?;

let result = check_consistency(&reasoner)?;
if !result.complete {
    // budget or tableau limit — do not treat as proof
}
if result.consistent {
    match classify(&mut reasoner)? {
        ClassifyOutcome::Taxonomy(t) => { /* use taxonomy */ }
        _ => unreachable!("Profile::Dl yields Taxonomy"),
    }
}
```

Direct engine access (tests, benchmarks):

```rust
use ontologos_dl::classify;

let taxonomy = classify(&ontology, budget_secs)?;
```

## Consistency

| API | Returns | When to use |
|-----|---------|-------------|
| `ontologos_facade::check_consistency` | `ConsistencyResult { consistent, complete }` | **Production** — inspect `complete` |
| `ontologos_facade::is_consistent` | `Result<bool>` | Scripts only; `Err` on incomplete checks |

When `complete == false`, the DL engine hit a wall-clock or tableau budget. Increase `budget_secs` or fail closed.

## Budget configuration

| Mechanism | Description |
|-----------|-------------|
| `ReasonerConfig::budget_secs` | Per-reasoner wall-clock limit (seconds) |
| `ONTOLOGOS_DL_BUDGET_SECS` | Env fallback when `budget_secs` is `None` |
| CLI `--budget-secs` | Global option on `classify` / consistency paths |

Without a budget, DL may run until natural completion — risky on pathological inputs.

## `Profile::Dl` vs `Profile::DlPreview`

| Profile | Status | Behavior |
|---------|--------|----------|
| `Dl` | Stable on v1.1.1 | Full DL engine |
| `DlPreview` | Preview | Same engine with explicit preview gating — see [Preview profiles](../guides/preview-profiles.md) |

## Hybrid routing (`Profile::Auto`)

Mixed EL/DL ontologies may route modules separately (MORe-style hybrid). CLI `profile` JSON may include `hybrid_modules`. Validate on your corpus — catalog parity does not cover every production ontology.

## Python

On PyPI **1.1.1**:

```python
from ontologos import Reasoner

reasoner = Reasoner(path="ontology.owl", profile="dl", budget_secs=30)
consistency = reasoner.check_consistency()
if not consistency["complete"]:
    raise RuntimeError("DL consistency incomplete")
report = reasoner.classify()
```

## CLI

```bash
ontologos classify --profile dl --budget-secs 30 ontology.owl
```

## Production checklist

1. Always use `check_consistency` — not `is_consistent` alone.
2. Set `budget_secs` for every DL request.
3. Do **not** set `ONTOLOGOS_CONFORMANCE`, `ONTOLOGOS_STRICT_TAXONOMY`, or `ONTOLOGOS_CI_PROMOTED_ONLY` in production.
4. Validate on your corpus — HermiT catalog parity applies to gated test corpora only.
5. Serialize OWL loads — horned-owl parsing uses a process-wide mutex.

See [Security](../security.md) and [Production integration](../guides/production-integration.md).

## Errors

DL errors surface as `ontologos_facade::Error::Dl` or `ontologos_dl::Error`:

| Variant | Meaning |
|---------|---------|
| `IncompleteConsistency` / `IncompleteReasoning` | Budget or tableau limit hit |
| `Inconsistent` | Proved inconsistent |
| `PreviewLimit` | Preview profile cap exceeded |
| `ResourceLimit` | Axiom/entity cap exceeded |
| `WrongProfile` | Ontology/profile mismatch |

Full table: [Error reference](errors.md).

## Related

- [Facade API](facade.md)
- [Core API](core.md)
- [Conformance](conformance.md)
- [Taxonomy tolerance](taxonomy-tolerance.md)
- [Performance](../guides/performance.md)
