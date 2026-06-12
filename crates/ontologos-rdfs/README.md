# ontologos-rdfs

RDFS TBox materialization for [OntoLogos](https://github.com/eddiethedean/ontologos): transitive `subClassOf` / `subPropertyOf`, and domain/range inheritance along the property hierarchy.

```rust
use ontologos_parser::load_ontology;
use ontologos_rdfs::RdfsEngine;

let mut ontology = load_ontology(path)?;
let report = RdfsEngine::new().materialize(&mut ontology)?;
println!("inferred {} axioms", report.inferred_total());
```

See the [workspace README](../../README.md) and [docs.rs](https://docs.rs/ontologos-rdfs).
