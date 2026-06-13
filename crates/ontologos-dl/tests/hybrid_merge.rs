//! DL hybrid merge and domain/range integration tests.

use ontologos_core::{Axiom, Ontology};
use ontologos_dl::{classify, DlClassifier};

#[test]
fn merge_preserves_el_equivalences() {
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
        .add_axiom(Axiom::EquivalentClasses(vec![a, b]))
        .unwrap();

    let taxonomy = classify(&ontology).expect("classify");
    assert!(
        !taxonomy.equivalences.is_empty() || taxonomy.is_subsumed(a, b),
        "expected equivalence cluster or mutual subsumption"
    );
}

#[test]
fn preview_mode_classifies_el_fragment() {
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
    let taxonomy = DlClassifier::new()
        .preview(true)
        .classify(&ontology)
        .expect("preview classify");
    assert!(taxonomy.is_subsumed(a, b));
}
