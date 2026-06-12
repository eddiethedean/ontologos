# ontologos-rdfs

RDFS TBox materialization for [OntoLogos](https://github.com/eddiethedean/ontologos): transitive `subClassOf` / `subPropertyOf`, and object-property domain/range inheritance along the property hierarchy.

**v0.3 scope:** TBox rules only. Does not expand `EquivalentClasses`, data properties, or `rdf:type` (ABox deferred to v1.6).

```rust
use ontologos_parser::load_ontology;
use ontologos_rdfs::RdfsEngine;

let mut ontology = load_ontology(path)?;
let report = RdfsEngine::new().materialize(&mut ontology)?;
println!("inferred {} axioms", report.inferred_total());
```

Via the reasoner facade:

```rust
use ontologos_core::{Profile, Reasoner};
use ontologos_rdfs::classify_reasoner;

let mut reasoner = Reasoner::builder()
    .profile(Profile::Rdfs)
    .build(ontology)?;
classify_reasoner(&mut reasoner)?;
```

See the [workspace README](../../README.md) and [docs.rs](https://docs.rs/ontologos-rdfs).
