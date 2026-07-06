# Classify in five minutes (Rust)


--8<-- "snippets/before-integrate-callout.md"
Run OWL **classification** from crates.io — no repository clone. Uses `ontologos-facade::classify`.

!!! info "The one rule for Rust integrators"
    `Reasoner` holds config and ontology state. **Always** call `ontologos_facade::classify(&mut reasoner)` —
    never `reasoner.classify()`. See [Rust integration contract](../guides/rust-integration-contract.md).

## Prerequisites

- Rust **1.88+** (`rustc --version` must be >= 1.88.0)
- OntoLogos crates — see [Release status](../project/release-status.md) for published **v1.0.0** pins

```bash
curl -L -o family.owl \
  https://raw.githubusercontent.com/eddiethedean/ontologos/main/benchmarks/data/family.owl
cargo new ontologos-classify-demo && cd ontologos-classify-demo
```

## Cargo.toml

```toml
[dependencies]
ontologos-core = "1.1.2"
ontologos-parser = "1.1.2"
ontologos-facade = "1.1.2"
```

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

For EL without the facade, use an **in-memory EL ontology** (Family.owl is RL-shaped — do not use it for EL demos):

```toml
# Add to Cargo.toml
ontologos-el = "1.1.2"
```

```rust
use ontologos_core::Ontology;
use ontologos_el::ElClassifier;

fn main() -> Result<(), ontologos_core::Error> {
    let ontology = Ontology::builder()
        .class("http://example.org/Food")?
        .class("http://example.org/Pizza")?
        .subclass_of("http://example.org/Pizza", "http://example.org/Food")?
        .build()?;
    let taxonomy = ElClassifier::new().classify(&ontology)?;
    println!("subsumptions: {}", taxonomy.subsumption_count());
    Ok(())
}
```

For Pizza EL file-based demos, clone the repo and run `./benchmarks/scripts/download.sh`. See [OWL EL classification](owl-el-classification.md).

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
