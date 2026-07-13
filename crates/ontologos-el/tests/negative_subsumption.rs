//! Negative EL subsumption checks — taxonomy must not invent spurious edges.

use ontologos_core::Ontology;
use ontologos_el::ElClassifier;

#[test]
fn el_does_not_infer_reverse_subclass() {
    let ontology = Ontology::builder()
        .class("http://example.org/A")
        .unwrap()
        .class("http://example.org/B")
        .unwrap()
        .subclass_of("http://example.org/A", "http://example.org/B")
        .unwrap()
        .build()
        .unwrap();
    let a = ontology.lookup_entity("http://example.org/A").unwrap();
    let b = ontology.lookup_entity("http://example.org/B").unwrap();
    let taxonomy = ElClassifier::new().classify(&ontology).expect("classify");
    assert!(taxonomy.is_subsumed(a, b));
    assert!(
        !taxonomy.is_subsumed(b, a),
        "EL must not infer B ⊑ A from A ⊑ B alone"
    );
}

#[test]
fn el_does_not_equate_sibling_classes() {
    let ontology = Ontology::builder()
        .class("http://example.org/A")
        .unwrap()
        .class("http://example.org/B")
        .unwrap()
        .class("http://example.org/C")
        .unwrap()
        .subclass_of("http://example.org/A", "http://example.org/C")
        .unwrap()
        .subclass_of("http://example.org/B", "http://example.org/C")
        .unwrap()
        .build()
        .unwrap();
    let a = ontology.lookup_entity("http://example.org/A").unwrap();
    let b = ontology.lookup_entity("http://example.org/B").unwrap();
    let taxonomy = ElClassifier::new().classify(&ontology).expect("classify");
    assert!(!taxonomy.is_subsumed(a, b));
    assert!(!taxonomy.is_subsumed(b, a));
}
