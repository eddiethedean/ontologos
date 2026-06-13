# Getting Started

Five-minute success paths for common goals.

## Crates.io only (no clone)

Use any OWL file on disk — no repository clone or benchmark download required.

```bash
cargo new ontologos-demo && cd ontologos-demo
```

Add to `Cargo.toml`:

```toml
[dependencies]
ontologos-core = "0.7.0"
ontologos-parser = "0.7.0"
ontologos-rdfs = "0.7.0"
```

`src/main.rs`:

```rust
use ontologos_parser::load_ontology;
use ontologos_rdfs::RdfsEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ontology = load_ontology(std::path::Path::new("ontology.owl"))?;
    let report = RdfsEngine::new().materialize(&mut ontology)?;
    println!(
        "inferred {} axioms ({} → {})",
        report.inferred_total(),
        report.initial_axiom_count,
        report.final_axiom_count
    );
    Ok(())
}
```

Place your OWL file at `ontology.owl`, then `cargo run`.

For OWL RL saturation, add `ontologos-rl = "0.7.0"` and see [OWL RL saturation](owl-rl-saturation.md).

## I want to try it from a clone

1. Clone and download benchmarks:

   ```bash
   git clone https://github.com/eddiethedean/ontologos.git
   cd ontologos
   ./benchmarks/scripts/download.sh
   ```

2. Run the builder example:

   ```bash
   cargo run -p ontologos-core --example pizza_builder
   ```

3. Build the CLI and inspect an ontology:

   ```bash
   cargo build -p ontologos-cli --release
   ./target/release/ontologos profile benchmarks/data/family.owl
   ./target/release/ontologos materialize benchmarks/data/family.owl
   ```

## I want RDFS materialization

Follow [RDFS materialization](rdfs-materialization.md). Prefer CLI **`materialize`** over **`classify`** — both run the same engine in v0.4.

## I want OWL RL saturation (v0.4 headline feature)

Follow [OWL RL saturation](owl-rl-saturation.md) or run:

```bash
cargo run -p ontologos-rl --example rl_saturation
```

OWL RL is **not** available in the CLI until v0.5 — use the library or Python `profile="rl"`.

## I'm integrating in Rust

Read [Choosing an API](../guides/choosing-an-api.md) then the guide for your workflow:

| Goal | Guide |
|------|-------|
| Build ontologies in code | [First ontology](first-ontology.md) |
| Load OWL files | [Load an OWL file](load-owl-file.md) |
| RDFS materialization | [RDFS materialization](rdfs-materialization.md) |
| OWL RL saturation | [OWL RL saturation](owl-rl-saturation.md) |
| JSON snapshots | [JSON snapshot v2](../json-snapshot-v2.md) |

## I'm evaluating vs ELK / reasonable

See [Comparison with existing tools](../comparison.md) and [Conformance coverage](../reference/conformance.md).

## I'm using Python

```bash
pip install ontologos
```

```python
from ontologos import Reasoner

# Always set profile= — default "auto" fails in v0.4
Reasoner("ontology.owl", profile="rdfs").classify()
```

See [Python guide](../guides/python.md).

## Full learning path

See the [documentation index](../index.md#learning-path).
