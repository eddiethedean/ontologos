# ontologos-facade

Unified OWL reasoner routing for OntoLogos — one `classify()` entry for EL, RL, RDFS, DL, ALC, and SWRL.

**Full guide:** [Facade API](https://ontologos.readthedocs.io/en/latest/guides/facade-api.html)

## Install

```toml
[dependencies]
ontologos-core = "1.1.4"
ontologos-parser = "1.1.4"
ontologos-facade = "1.1.4"
```

Bump all `ontologos-*` pins together. Published on crates.io: **1.1.4** — see [Release status](https://ontologos.readthedocs.io/en/latest/project/release-status/).

## Quick start

Download `family.owl` first:

```bash
curl -L -o family.owl \
  https://raw.githubusercontent.com/eddiethedean/ontologos/main/benchmarks/data/family.owl
```

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
