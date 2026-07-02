# Classify in five minutes (Rust)

Run OWL **classification** from crates.io — no repository clone. Uses `ontologos-facade::classify`.

!!! tip "Rust API in 60 seconds"
    Build `Reasoner` with `Reasoner::builder()` → call **`ontologos_facade::classify(&mut reasoner)`**. Do **not** call `ontologos_core::Reasoner::classify()`. See [Facade API](../guides/facade-api.md).

## Prerequisites

- Rust **1.88+**
- OntoLogos crates — see [Release status](../project/release-status.md) for published vs `main` pins

```bash
curl -L -o family.owl \
  https://raw.githubusercontent.com/eddiethedean/ontologos/main/benchmarks/data/family.owl
cargo new ontologos-classify-demo && cd ontologos-classify-demo
```

## Cargo.toml

```toml
[dependencies]
ontologos-core = "0.9.0"
ontologos-parser = "0.9.0"
ontologos-facade = "0.9.0"
```

Build from `main`? Pin `"1.0.0"` on all `ontologos-*` crates instead.

## Classify with profile auto

```rust
use ontologos_core::{Profile, Reasoner};
use ontologos_facade::ClassifyOutcome;
use ontologos_parser::load_ontology;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ontology = load_ontology(std::path::Path::new("family.owl"))?;
    let mut reasoner = Reasoner::builder()
        .profile(Profile::Auto)
        .build(ontology)?;
    match ontologos_facade::classify(&mut reasoner)? {
        ClassifyOutcome::Taxonomy(t) => {
            println!("subsumptions: {}", t.subsumption_count());
        }
        ClassifyOutcome::Rdfs(r) => {
            println!("inferred: {}", r.inferred_total());
        }
        ClassifyOutcome::Rl(r) => {
            println!("inferred: {}", r.inferred_total());
        }
    }
    Ok(())
}
```

`Profile::Auto` detects RL for Family and runs saturation. **Expected:** `inferred` > 0; ~57 mapped axioms (normal for Family — not Protégé totals). See [Protégé axiom counts](../guides/protege-axiom-counts.md).

For EL taxonomies, use `Profile::El` with an EL-shaped ontology (in-memory builder below, or Pizza after clone + `./benchmarks/scripts/download.sh`).

## Classify with a profile crate directly

For a single engine without the facade:

```rust
use ontologos_el::ElClassifier;
use ontologos_parser::load_ontology;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ontology = load_ontology("ontology.owl".as_ref())?;
    let taxonomy = ElClassifier::new().classify(&ontology)?;
    println!("subsumptions: {}", taxonomy.subsumption_count());
    Ok(())
}
```

## Do not use

```rust
// WRONG — deprecated; returns NotImplemented / delegate hints
ontologos_core::Reasoner::classify(&mut reasoner)?;
```

See [Choosing an API](../guides/choosing-an-api.md).

## Next steps

- [Known limitations](../guides/known-limitations.md) — imports, mapping, axiom counts
- [Load an OWL file](load-owl-file.md) — formats, `ParseMeta`
- [Profile stability matrix](../guides/profile-stability.md) — which profiles are production-ready
- [Examples gallery](../examples/index.md) — more copy-paste workflows
