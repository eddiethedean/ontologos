# OWL RL Saturation

OWL 2 RL forward-chaining via [`ontologos-rl`](https://docs.rs/ontologos-rl/0.9.0). The engine runs RDFS materialization first, then applies RL TBox and ABox rules until a fixed point.

## Prerequisites

- Rust 1.88+
- Clone of the [OntoLogos repository](https://github.com/eddiethedean/ontologos) (Family corpus is vendored; no download required for this walkthrough)

## Run the example

```bash
cargo run -p ontologos-rl --example rl_saturation
```

## End-to-end: Family ontology

The Family benchmark is an OWL RL corpus with ABox assertions. Profile detection typically reports `Rl`.

### CLI

```bash
cargo build -p ontologos-cli --release
./target/release/ontologos profile benchmarks/data/family.owl
./target/release/ontologos classify --profile rl benchmarks/data/family.owl
```

Profile detection output (abbreviated):

```text
detected profile: Rl
```

`classify --profile rl` runs OWL RL saturation and prints subsumption counts or a materialization report. Use `--format json` for machine-readable output. See [CLI reference](../reference/cli.md).

### Library

Add dependencies:

```toml
[dependencies]
ontologos-core = "0.9.0"
ontologos-parser = "0.9.0"
ontologos-rl = "0.9.0"
```

Load and saturate:

```rust
use ontologos_parser::load_ontology;
use ontologos_rl::RlEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("benchmarks/data/family.owl");
    let mut ontology = load_ontology(path)?;

    let initial = ontology.axiom_count();
    let report = RlEngine::new(1).saturate(&mut ontology)?;

    println!("initial axioms: {initial}");
    println!("final axioms: {}", report.final_axiom_count);
    println!("inferred total: {}", report.inferred_total());
    println!("RDFS inferred: {}", report.rdfs_inferred);
    for (rule, count) in &report.inferred_by_rule {
        println!("  {}: {count}", rule.as_str());
    }
    if !report.clashes.is_empty() {
        for clash in &report.clashes {
            println!("clash: {clash}");
        }
    }

    Ok(())
}
```

### Via the reasoner facade

```rust
use ontologos_core::{Profile, Reasoner};
use ontologos_parser::load_ontology;
use ontologos_rl::classify_reasoner;

let ontology = load_ontology(path)?;
let mut reasoner = Reasoner::builder()
    .profile(Profile::Rl)
    .build(ontology)?;

classify_reasoner(&mut reasoner)?;
let ontology = reasoner.ontology();
println!("axioms after saturation: {}", ontology.axiom_count());
```

Do **not** call `Reasoner::classify()` on core directly for RL — it returns a delegate hint. Use `ontologos_rl::classify_reasoner` or `RlEngine::saturate`.

## Reading the report

`MaterializationReport` fields:

| Field | Meaning |
|-------|---------|
| `initial_axiom_count` | Axioms before saturation |
| `final_axiom_count` | Axioms after RDFS + RL |
| `rdfs_inferred` | Axioms added during the RDFS pass |
| `inferred_by_rule` | RL rule counts (see [RL rules reference](../reference/rl-rules.md)) |
| `clashes` | Detected inconsistencies (disjoint types, `sameAs`/`differentFrom` conflicts) |
| `traces` | Per-inference records when `RlEngine::with_traces(true)` is set |

`inferred_total()` equals `final_axiom_count - initial_axiom_count` (includes both RDFS and RL inferences).

## Parallelism

`RlEngine::new(n)` accepts `parallelism` in `1..=64`. Values outside that range return an error via `RlEngine::try_new`. Parallelism affects ABox type-rule candidate expansion only; use `1` for fully sequential execution.

```rust
let report = RlEngine::try_new(4)?.saturate(&mut ontology)?;
```

## Python

```python
from ontologos import Reasoner

reasoner = Reasoner("benchmarks/data/family.owl", profile="rl")
reasoner.classify()
meta = reasoner.parse_meta
print(meta["mapped_axiom_count"])
```

See [Python guide](../guides/python.md) for limitations.

## What RL adds over RDFS

| Capability | RDFS only | OWL RL |
|------------|-----------|--------|
| Transitive `subClassOf` / `subPropertyOf` | Yes | Yes (via RDFS pass) |
| Domain/range inheritance | Yes | Yes |
| `EquivalentClasses` → mutual subsumption | No | Yes |
| Property characteristics propagation | No | Yes |
| Existential subsumption | No | Yes |
| ABox type/property propagation | No | Yes |
| `sameAs` propagation | No | Yes |
| Clash detection | No | Partial (see [supported constructs](../reference/supported-constructs.md)) |

## Next steps

- [Choosing an API](../guides/choosing-an-api.md) — when to use RDFS vs RL vs EL
- [OWL EL classification](owl-el-classification.md) — completion-based taxonomy
- [RL rules reference](../reference/rl-rules.md) — rule names and meanings
- [Profile detection](../guides/profile-detection.md) — confirm your ontology is RL before saturating
- [Troubleshooting](../guides/troubleshooting.md) — common issues
