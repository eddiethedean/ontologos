# Getting Started

Five-minute success paths for common goals. Install pins: [Install and channels](../guides/install-channels.md). Limitations: [Known limitations](../guides/known-limitations.md).

## Rust API in 60 seconds

1. Load with `ontologos_parser::load_ontology` (not `Ontology::from_file`).
2. Build a `Reasoner` with `Reasoner::builder().profile(...).build(ontology)`.
3. Call **`ontologos_facade::classify(&mut reasoner)`** — not `reasoner.classify()` on core.

```rust
use ontologos_core::{Profile, Reasoner};
use ontologos_facade::{classify, ClassifyOutcome};
use ontologos_parser::load_ontology;

let ontology = load_ontology("family.owl".as_ref())?;
let mut reasoner = Reasoner::builder().profile(Profile::Auto).build(ontology)?;
match classify(&mut reasoner)? {
    ClassifyOutcome::Rl(r) => println!("inferred: {}", r.inferred_total()),
    // ...
}
```

See [Classify quick start](classify-quickstart.md) and [Facade API](../guides/facade-api.md).

## Crates.io only (no clone)

Download a sample ontology, then build a minimal Rust project:

```bash
curl -L -o family.owl \
  https://raw.githubusercontent.com/eddiethedean/ontologos/main/benchmarks/data/family.owl
cargo new ontologos-demo && cd ontologos-demo
```

Add to `Cargo.toml`:

```toml
[dependencies]
ontologos-core = "0.9.0"
ontologos-parser = "0.9.0"
ontologos-rdfs = "0.9.0"
```

`src/main.rs`:

```rust
use ontologos_parser::load_ontology;
use ontologos_rdfs::RdfsEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ontology = load_ontology(std::path::Path::new("family.owl"))?;
    let report = RdfsEngine::new().materialize(&mut ontology)?;
    println!(
        "mapped {} → {} axioms (inferred {})",
        report.initial_axiom_count,
        report.final_axiom_count,
        report.inferred_total()
    );
    Ok(())
}
```

Then `cargo run`.

**Expected output (family.owl):** mapped axioms ~57; `inferred` > 0. Counts differ from Protégé — see [Known limitations](../guides/known-limitations.md).

For OWL RL saturation, add `ontologos-rl = "0.9.0"` and see [OWL RL saturation](owl-rl-saturation.md).

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

Follow [RDFS materialization](rdfs-materialization.md). Prefer CLI **`materialize`** over **`classify --profile rdfs`** — both run the same RDFS engine.

## I want OWL RL saturation

Follow [OWL RL saturation](owl-rl-saturation.md) or run:

```bash
cargo run -p ontologos-rl --example rl_saturation
```

From a clone with the CLI built:

```bash
./target/release/ontologos classify --profile rl benchmarks/data/family.owl
```

Or use Python: `Reasoner(path="family.owl", profile="rl").classify()`.

## I'm integrating in Rust

Read [Choosing an API](../guides/choosing-an-api.md) then the guide for your workflow:

| Goal | Guide |
|------|-------|
| Build ontologies in code | [First ontology](first-ontology.md) |
| Load OWL files | [Load an OWL file](load-owl-file.md) |
| RDFS materialization | [RDFS materialization](rdfs-materialization.md) |
| OWL RL saturation | [OWL RL saturation](owl-rl-saturation.md) |
| OWL EL classification | [OWL EL classification](owl-el-classification.md) |
| JSON snapshots | [JSON snapshot v3](../json-snapshot-v3.md) ([v2 legacy](../json-snapshot-v2.md)) |

## I'm evaluating vs ELK / reasonable

See [Comparison with existing tools](../comparison.md) and [Conformance coverage](../reference/conformance.md).

## I'm using Python

```bash
pip install ontologos
curl -L -o family.owl \
  https://raw.githubusercontent.com/eddiethedean/ontologos/main/benchmarks/data/family.owl
```

```python
from ontologos import Reasoner, OntologyBuilder

# profile defaults to "auto" (EL taxonomy or RL saturation)
r = Reasoner(path="family.owl")
report = r.classify()

# In-memory ontology + incremental edits
b = OntologyBuilder()
b.add_class("http://example.org/Food")
b.add_class("http://example.org/Pizza")
b.subclass_of("http://example.org/Pizza", "http://example.org/Food")
r = Reasoner(ontology=b.build(), profile="el", incremental=True)
r.classify()
r.add_subclass_of("http://example.org/VeggiePizza", "http://example.org/Pizza")
r.classify()
```

See [Python guide](../guides/python.md) and [Known limitations](../guides/known-limitations.md).

## Full learning path

See the [documentation index](../index.md#learning-path).

## Classify from Rust (no clone)

[Classify quick start](classify-quickstart.md) — `ontologos-facade::classify` in five minutes.
