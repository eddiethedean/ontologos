//! DL classifier integration smoke tests.

use ontologos_core::Ontology;
use ontologos_dl::classify;

#[test]
fn classifies_named_subsumption_via_tableau() {
    let ontology = Ontology::builder()
        .class("http://ex/A")
        .unwrap()
        .class("http://ex/B")
        .unwrap()
        .subclass_of("http://ex/A", "http://ex/B")
        .unwrap()
        .build()
        .unwrap();
    let a = ontology.lookup_entity("http://ex/A").unwrap();
    let b = ontology.lookup_entity("http://ex/B").unwrap();
    let taxonomy = classify(&ontology).expect("classify");
    assert!(taxonomy.is_subsumed(a, b));
}

#[test]
fn detects_disjoint_unsatisfiable_class() {
    let mut ontology = Ontology::builder()
        .class("http://ex/A")
        .unwrap()
        .class("http://ex/B")
        .unwrap()
        .build()
        .unwrap();
    let a = ontology.lookup_entity("http://ex/A").unwrap();
    let b = ontology.lookup_entity("http://ex/B").unwrap();
    ontology
        .add_axiom(ontologos_core::Axiom::DisjointClasses(vec![a, b]))
        .unwrap();
    ontology
        .add_axiom(ontologos_core::Axiom::EquivalentClasses(vec![a, b]))
        .unwrap();
    let taxonomy = classify(&ontology).expect("classify");
    assert!(taxonomy.unsatisfiable.contains(&a) || taxonomy.unsatisfiable.contains(&b));
}
