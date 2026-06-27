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

#[test]
fn family_relative_union_members_subsumed() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data/family.owl");
    let ontology = ontologos_parser::load_ontology(&path).expect("load family");
    let taxonomy = classify(&ontology).expect("classify");
    let ns = "http://a.com/ontology#";
    let child = ontology.lookup_entity(&format!("{ns}Child")).unwrap();
    let parent = ontology.lookup_entity(&format!("{ns}Parent")).unwrap();
    let sibling = ontology.lookup_entity(&format!("{ns}Sibling")).unwrap();
    let relative = ontology.lookup_entity(&format!("{ns}Relative")).unwrap();
    assert!(taxonomy.is_subsumed(child, relative), "Child ⊑ Relative");
    assert!(taxonomy.is_subsumed(parent, relative), "Parent ⊑ Relative");
    assert!(
        taxonomy.is_subsumed(sibling, relative),
        "Sibling ⊑ Relative"
    );
}
