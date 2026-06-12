# ontologos-rdfs

RDFS TBox materialization for [OntoLogos](https://github.com/eddiethedean/ontologos): transitive `subClassOf` / `subPropertyOf`, and object-property domain/range inheritance along the property hierarchy.

**v0.4 scope:** TBox rules only. Does not expand `EquivalentClasses` or data properties. ABox `rdf:type` propagation and equivalent-class expansion are handled by [`ontologos-rl`](../ontologos-rl).

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

See the [workspace README](../../README.md), [documentation site](https://ontologos.readthedocs.io/en/latest/), and [docs.rs](https://docs.rs/ontologos-rdfs).
