# ontologos-facade

Unified OWL reasoner routing for OntoLogos — one `classify()` entry for EL, RL, RDFS, DL, ALC, and SWRL.

**Full guide:** [Facade API](https://ontologos.readthedocs.io/en/latest/guides/facade-api.html)

## Install

```toml
[dependencies]
ontologos-core = "0.9.0"
ontologos-parser = "0.9.0"
ontologos-facade = "0.9.0"
```

Bump all `ontologos-*` pins together. On `main`, use `"1.0.0"` for DL and the full engine set.

## Quick start

```rust
use ontologos_core::{Profile, Reasoner};
use ontologos_facade::ClassifyOutcome;
use ontologos_parser::load_ontology;

let ontology = load_ontology("family.owl".as_ref())?;
let mut reasoner = Reasoner::builder()
    .profile(Profile::Auto)
    .build(ontology)?;

match ontologos_facade::classify(&mut reasoner)? {
    ClassifyOutcome::Taxonomy(t) => println!("{}", t.subsumption_count()),
    ClassifyOutcome::Rdfs(r) => println!("{}", r.inferred_total()),
    ClassifyOutcome::Rl(r) => println!("{}", r.inferred_total()),
}
```

## Example binary

From a clone:

```bash
cargo run -p ontologos-facade --example facade_auto -- benchmarks/data/family.owl
```

## Related

- [Choosing an API](https://ontologos.readthedocs.io/en/latest/guides/choosing-an-api.html)
- [Profile stability](https://ontologos.readthedocs.io/en/latest/guides/profile-stability.html)
