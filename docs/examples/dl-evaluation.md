# DL evaluation example

OWL 2 DL classification on workspace **1.0.0** (`main`). Not available on PyPI **0.9.0**.

Install: [Install channels](../guides/install-channels.md) · API: [DL reference](../reference/dl.md).

## Prerequisites

- Rust **1.88+**
- Clone and download corpora:

```bash
git clone https://github.com/eddiethedean/ontologos.git
cd ontologos
./benchmarks/scripts/download.sh
cargo build -p ontologos-cli --release
```

## CLI

```bash
./target/release/ontologos --format json classify \
  --profile dl --budget-secs 30 benchmarks/data/pizza.owl
```

**Expected:** `status: "classified"`, `subsumption_count` > 0, consistency `complete: true` when run via `consistent` subcommand first.

```bash
./target/release/ontologos --format json consistent \
  --budget-secs 30 benchmarks/data/pizza.owl
```

## Rust (facade)

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
    .build(load_ontology("benchmarks/data/pizza.owl".as_ref())?)?;

let result = check_consistency(&reasoner)?;
assert!(result.complete && result.consistent);

match classify(&mut reasoner)? {
    ClassifyOutcome::Taxonomy(t) => println!("subsumptions: {}", t.subsumption_count()),
    _ => unreachable!(),
}
```

## Python

Build `ontologos-py` from `main` (`maturin develop`) or wait for PyPI **1.0.0**:

```python
from ontologos import Reasoner

reasoner = Reasoner(
    path="benchmarks/data/pizza.owl",
    profile="dl",
    budget_secs=30,
)
consistency = reasoner.check_consistency()
if not consistency["complete"]:
    raise RuntimeError("increase budget_secs")
report = reasoner.classify()
print(report["subsumption_count"])
```

## Related

- [Evaluator playbook](../guides/evaluator-playbook.md)
- [Evaluator scope](../guides/evaluator-scope.md)
- [Production integration — OWL DL](../guides/production-integration.md#owl-dl-in-production)
- [Examples gallery](index.md)
