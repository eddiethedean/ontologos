# SWRL quick start


--8<-- "snippets/before-integrate-callout.md"
Run **DLSafe SWRL** rules with OWL 2 DL on **v1.1.3** (stable on crates.io and PyPI).

Profile matrix: [Profile stability](../guides/profile-stability.md). Constructs: [Supported constructs](../reference/supported-constructs.md).

## Prerequisites

- Rust **1.88+** or Python **3.10+**
- An ontology that includes SWRL rules in a supported serialization (RDF/XML or OWL Functional)

SWRL support is **DLSafe** — rules must not bind variables to anonymous individuals in unsafe positions. Unsupported rules are skipped with parser warnings.

## Python

```bash
pip install ontologos==1.1.3
```

```python
from ontologos import Reasoner

# Replace with your ontology path containing SWRL rules
reasoner = Reasoner(path="my-rules.owl", profile="swrl", budget_secs=30)
consistency = reasoner.check_consistency()
print("consistent:", consistency["consistent"], "complete:", consistency["complete"])

if consistency["complete"] and consistency["consistent"]:
    report = reasoner.classify()
    print(report)
```

Inspect load warnings:

```python
print(reasoner.parse_meta.get("warnings", []))
```

## Rust (facade)

```toml
[dependencies]
ontologos-core = "1.1.3"
ontologos-parser = "1.1.3"
ontologos-facade = "1.1.3"
```

```rust
use ontologos_core::{Profile, Reasoner, ReasonerConfig};
use ontologos_facade::{check_consistency, classify};
use ontologos_parser::load_ontology;

let ontology = load_ontology("my-rules.owl".as_ref())?;
let mut reasoner = Reasoner::builder()
    .profile(Profile::Swrl)
    .config(ReasonerConfig {
        budget_secs: Some(30),
        ..ReasonerConfig::default()
    })
    .build(ontology)?;

let consistency = check_consistency(&reasoner)?;
if consistency.complete && consistency.consistent {
    classify(&mut reasoner)?;
}
```

See [Rust integration contract](../guides/rust-integration-contract.md).

## CLI

```bash
cargo install --git https://github.com/eddiethedean/ontologos --tag v1.1.3 ontologos-cli
ontologos classify --profile swrl --budget-secs 30 my-rules.owl
```

Install details: [CLI installation](../getting-started/cli-install.md).

## What to expect

| Step | Behavior |
|------|----------|
| Load | Rules mapped via `map_swrl.rs`; unsafe rules may be skipped |
| Consistency | DL + rule grounding; check `complete` before trusting `consistent` |
| Classify | Materialized rule consequences + DL taxonomy where applicable |

## Troubleshooting

| Issue | Action |
|-------|--------|
| Rules ignored | Check `parse_meta.warnings`; verify DLSafe shape |
| `IncompleteConsistency` | Increase `budget_secs` or `ONTOLOGOS_DL_BUDGET_SECS` |
| Wrong profile | Run `ontologos profile file.owl` — SWRL ontologies often detect as DL |

## Related

- [SWRL API reference](../reference/swrl.md)
- [DL API reference](../reference/dl.md)
- [Errors](../reference/errors.md)
