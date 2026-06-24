# RDFS Materialization

RDFS TBox materialization via [`ontologos-rdfs`](https://docs.rs/ontologos-rdfs/0.9.0): transitive `subClassOf` / `subPropertyOf` closure and object-property domain/range inheritance.

The engine delegates to [`reasonable`](https://crates.io/crates/reasonable) and applies bridge fallbacks for RDFS rules not yet upstream (transitive `subPropertyOf`, domain/range along property hierarchies). You may therefore see OWL RL-style inferences beyond strict RDFS on some corpora.

## Prerequisites

- Rust 1.88+
- An OWL/RDF file (`.owl`, `.rdf`, `.ttl`, `.ofn`) or a repository clone for benchmark examples

## Run the CLI

Build from a clone (CLI is not on crates.io):

```bash
cargo build -p ontologos-cli --release
./target/release/ontologos materialize path/to/ontology.owl
```

Prefer **`materialize`** over **`classify --profile rdfs`** — both run the same RDFS engine, but `classify` is named like OWL taxonomy classification tools (ELK/HermiT) and is easy to misread.

Expected text output (Family corpus, abbreviated):

```text
status: materialized
initial_axiom_count: 57
final_axiom_count: 62
inferred_axioms: 5
inferred_by_rule: none
```

> **Adapter note:** `ontologos-rdfs` delegates to **reasonable**. `inferred_by_rule` is empty until reasonable exposes rule-level diagnostics; use `inferred_axioms` and axiom counts. Some RDFS rules (e.g. transitive `subPropertyOf`) have known upstream gaps — see [dependency-first ADR](../internal/design/dependency-first.md).

JSON output: `./target/release/ontologos --format json materialize path/to/ontology.owl`

## Library (crates.io)

Add dependencies:

```toml
[dependencies]
ontologos-core = "0.9.0"
ontologos-parser = "0.9.0"
ontologos-rdfs = "0.9.0"
```

Load and materialize:

```rust
use ontologos_parser::load_ontology;
use ontologos_rdfs::RdfsEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("ontology.owl");
    let mut ontology = load_ontology(path)?;

    let initial = ontology.axiom_count();
    let report = RdfsEngine::new().materialize(&mut ontology)?;

    println!("initial axioms: {initial}");
    println!("final axioms: {}", report.final_axiom_count);
    println!("inferred: {}", report.inferred_total());
    for (rule, count) in &report.inferred_by_rule {
        println!("  {}: {count}", rule.as_str());
    }

    Ok(())
}
```

Do **not** call `Reasoner::classify()` on core for RDFS — it returns a delegate hint. Use `RdfsEngine::materialize` or `ontologos_rdfs::classify_reasoner`.

### Via the reasoner facade

```rust
use ontologos_core::{Profile, Reasoner};
use ontologos_parser::load_ontology;
use ontologos_rdfs::classify_reasoner;

let ontology = load_ontology(path)?;
let mut reasoner = Reasoner::builder()
    .profile(Profile::Rdfs)
    .build(ontology)?;

classify_reasoner(&mut reasoner)?;
println!("axioms: {}", reasoner.ontology().axiom_count());
```

## Reading the report

| Field | Meaning |
|-------|---------|
| `initial_axiom_count` | Axioms before materialization |
| `final_axiom_count` | Axioms after RDFS closure |
| `inferred_by_rule` | Per-rule counts (`subclass_trans`, `subprop_trans`, `dom_inherit`, `rng_inherit`) |
| `traces` | Per-inference records when tracing is enabled |

`inferred_total()` equals `final_axiom_count - initial_axiom_count`.

## What RDFS materializes

| Input in core | Materialized | Via reasonable |
|---------------|--------------|----------------|
| `SubClassOf` | Transitive closure | Yes |
| `SubObjectPropertyOf` | Transitive closure | **Gap** — see [Reasonable adapter limits](../reference/reasonable-limits.md) |
| `ObjectPropertyDomain` / `ObjectPropertyRange` | Inherited along `subPropertyOf` | **Gap** — see [Reasonable adapter limits](../reference/reasonable-limits.md) |
| `EquivalentClasses` | Stored only — mutual subsumption via [OWL RL saturation](../getting-started/owl-rl-saturation.md) | Partial |
| ABox assertions | Not propagated — use RL saturation | — |

## Python

```python
from ontologos import Reasoner

reasoner = Reasoner(path="ontology.owl", profile="rdfs")
reasoner.classify()
print(reasoner.parse_meta["mapped_axiom_count"])
```

See [Python guide](../guides/python.md) for limitations.

## Next steps

- [OWL RL saturation](owl-rl-saturation.md) — adds equivalence, ABox propagation, and more
- [Choosing an API](../guides/choosing-an-api.md) — RDFS vs RL vs EL
- [Profile detection](../guides/profile-detection.md)
- [Troubleshooting](../guides/troubleshooting.md)
