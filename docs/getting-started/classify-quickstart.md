# Classify in five minutes (Rust)

Run OWL **classification** from crates.io — no repository clone. Uses `ontologos-facade` (not the `ontologos_core::Reasoner` stub).

## Prerequisites

- Rust **1.88+**
- Published crates **0.9.0** on crates.io (see [Release status](../project/release-status.md))

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

Tracking `main` (1.0.0 workspace)? Use `"1.0.0"` pins instead and build from git.

## Classify with profile auto

```rust
use ontologos_core::{Profile, Reasoner};
use ontologos_parser::load_ontology;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ontology = load_ontology(std::path::Path::new("family.owl"))?;
    let mut reasoner = Reasoner::builder()
        .profile(Profile::Auto)
        .build(ontology)?;
    let outcome = ontologos_facade::classify(&mut reasoner)?;
    println!("profile: {:?}", outcome.profile);
    println!("status: {:?}", outcome.status);
    Ok(())
}
```

`Profile::Auto` detects RL for Family and runs saturation. For EL taxonomies, use `Profile::El` or load a Pizza-shaped ontology (clone + `./benchmarks/scripts/download.sh`).

## Classify with a profile crate directly

For a single engine without the facade:

```rust
use ontologos_el::ElClassifier;
use ontologos_parser::load_ontology;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ontology = load_ontology("ontology.owl".as_ref())?;
    let taxonomy = ElClassifier::new().classify(&ontology)?;
    println!("subsumptions: {}", taxonomy.subsumptions.len());
    Ok(())
}
```

## Do not use

```rust
// WRONG — returns NotImplemented / delegate hints
ontologos_core::Reasoner::classify(&mut reasoner)?;
```

See [Choosing an API](../guides/choosing-an-api.md).

## Next steps

- [Load an OWL file](load-owl-file.md) — formats, `ParseMeta`, imports limitation
- [Profile stability matrix](../guides/profile-stability.md) — which profiles are production-ready
- [Examples gallery](../examples/index.md) — more copy-paste workflows
