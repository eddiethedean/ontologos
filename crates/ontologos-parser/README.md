# ontologos-parser

OWL/RDF file loading for OntoLogos (`load_ontology`).

**Guide:** [Load an OWL file](https://ontologos.readthedocs.io/en/latest/getting-started/load-owl-file.html)

## Install

```toml
[dependencies]
ontologos-parser = "1.1.1"
```

## Quick start

```rust
use ontologos_parser::load_ontology;

let ontology = load_ontology("ontology.owl".as_ref())?;
println!("entities: {}", ontology.entity_count());
```

`Ontology::from_file` on `ontologos-core` intentionally returns `ParseNotAvailable` — always use this crate for files.

## Example

```bash
cargo run -p ontologos-parser --example load_and_profile
```
