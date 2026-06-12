# ontologos-rl

OWL 2 RL forward-chaining rule engine for [OntoLogos](https://github.com/eddiethedean/ontologos).

Runs RDFS materialization first, then applies OWL RL TBox and ABox rules until saturation.

Repository example:

```bash
cargo run -p ontologos-rl --example rl_saturation
```

See [OWL RL saturation guide](../../docs/getting-started/owl-rl-saturation.md).

```rust
use ontologos_core::Ontology;
use ontologos_rl::RlEngine;

let mut ontology = Ontology::builder()
    .class("http://example.org/A")?
    .class("http://example.org/B")?
    .object_property("http://example.org/R")?
    .object_property("http://example.org/S")?
    .subproperty_of("http://example.org/R", "http://example.org/S")?
    .build()?;

let report = RlEngine::new(1).saturate(&mut ontology)?;
assert!(report.inferred_total() >= 0);
# Ok::<(), Box<dyn std::error::Error>>(())
```

For [`Reasoner`](https://docs.rs/ontologos-core/latest/ontologos_core/struct.Reasoner.html) integration use `ontologos_rl::classify_reasoner`.

Documentation: [ontologos.readthedocs.io](https://ontologos.readthedocs.io/en/latest/) · [docs.rs/ontologos-rl](https://docs.rs/ontologos-rl)
