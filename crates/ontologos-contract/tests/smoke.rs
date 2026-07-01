//! Facade integration smoke tests (user API contract).

use ontologos_core::{Axiom, EntityKind, Ontology, Profile, Reasoner};
use ontologos_facade::{
    ClassifyOutcome, EntailmentCheck, classify, get_object_property_values,
    get_sub_object_properties, is_consistent, is_entailed_axiom, is_subsumption_entailed,
    taxonomy_from_outcome,
};

fn el_chain_ontology() -> Ontology {
    Ontology::builder()
        .class("http://example.org/A")
        .unwrap()
        .class("http://example.org/B")
        .unwrap()
        .class("http://example.org/C")
        .unwrap()
        .subclass_of("http://example.org/A", "http://example.org/B")
        .unwrap()
        .subclass_of("http://example.org/B", "http://example.org/C")
        .unwrap()
        .build()
        .unwrap()
}

#[test]
fn classify_el_subsumption_chain() {
    let mut reasoner = Reasoner::builder()
        .profile(Profile::El)
        .build(el_chain_ontology())
        .unwrap();
    let outcome = classify(&mut reasoner).unwrap();
    let tax = taxonomy_from_outcome(&outcome).expect("EL taxonomy");
    let a = reasoner
        .ontology()
        .lookup_entity("http://example.org/A")
        .unwrap();
    let c = reasoner
        .ontology()
        .lookup_entity("http://example.org/C")
        .unwrap();
    assert!(tax.is_subsumed(a, c));
    assert!(
        is_subsumption_entailed(
            &mut reasoner,
            "http://example.org/A",
            "http://example.org/C"
        )
        .unwrap()
    );
}

#[test]
fn classify_rl_detects_disjoint_clash() {
    let mut ontology = Ontology::builder()
        .class("http://example.org/A")
        .unwrap()
        .class("http://example.org/B")
        .unwrap()
        .class("http://example.org/D")
        .unwrap()
        .individual("http://example.org/x")
        .unwrap()
        .class_assertion("http://example.org/x", "http://example.org/B")
        .unwrap()
        .class_assertion("http://example.org/x", "http://example.org/D")
        .unwrap()
        .build()
        .unwrap();
    let a = ontology.lookup_entity("http://example.org/A").unwrap();
    let b = ontology.lookup_entity("http://example.org/B").unwrap();
    let d = ontology.lookup_entity("http://example.org/D").unwrap();
    ontology
        .add_axiom(Axiom::EquivalentClasses(vec![a, b]))
        .unwrap();
    ontology
        .add_axiom(Axiom::DisjointClasses(vec![a, d]))
        .unwrap();
    let reasoner = Reasoner::builder()
        .profile(Profile::Rl)
        .build(ontology)
        .unwrap();
    assert!(!is_consistent(&reasoner).unwrap());
}

#[test]
fn classify_dl_named_subsumption() {
    let ontology = Ontology::builder()
        .class("http://example.org/A")
        .unwrap()
        .class("http://example.org/B")
        .unwrap()
        .subclass_of("http://example.org/A", "http://example.org/B")
        .unwrap()
        .build()
        .unwrap();
    let mut reasoner = Reasoner::builder()
        .profile(Profile::Dl)
        .build(ontology)
        .unwrap();
    let outcome = classify(&mut reasoner).unwrap();
    assert!(taxonomy_from_outcome(&outcome).is_some());
    assert!(
        is_subsumption_entailed(
            &mut reasoner,
            "http://example.org/A",
            "http://example.org/B"
        )
        .unwrap()
    );
}

#[test]
fn is_entailed_class_assertion_via_subsumption() {
    let ontology = Ontology::builder()
        .class("http://example.org/A")
        .unwrap()
        .class("http://example.org/B")
        .unwrap()
        .individual("http://example.org/x")
        .unwrap()
        .subclass_of("http://example.org/A", "http://example.org/B")
        .unwrap()
        .class_assertion("http://example.org/x", "http://example.org/A")
        .unwrap()
        .build()
        .unwrap();
    let mut reasoner = Reasoner::builder()
        .profile(Profile::El)
        .build(ontology)
        .unwrap();
    assert!(
        is_entailed_axiom(
            &mut reasoner,
            EntailmentCheck::ClassAssertion {
                individual: "http://example.org/x".into(),
                class: "http://example.org/B".into(),
            }
        )
        .unwrap()
    );
}

#[test]
fn lookup_object_property_values_after_rl() {
    let ontology = Ontology::builder()
        .individual("http://example.org/c")
        .unwrap()
        .individual("http://example.org/d")
        .unwrap()
        .object_property("http://example.org/r")
        .unwrap()
        .object_property_assertion(
            "http://example.org/c",
            "http://example.org/r",
            "http://example.org/d",
        )
        .unwrap()
        .build()
        .unwrap();
    let reasoner = Reasoner::builder()
        .profile(Profile::Rl)
        .build(ontology)
        .unwrap();
    let values =
        get_object_property_values(&reasoner, "http://example.org/c", "http://example.org/r")
            .unwrap();
    assert_eq!(values, vec!["http://example.org/d"]);
}

#[test]
fn lookup_sub_object_properties_el() {
    let ontology = Ontology::builder()
        .object_property("http://example.org/p")
        .unwrap()
        .object_property("http://example.org/q")
        .unwrap()
        .subproperty_of("http://example.org/q", "http://example.org/p")
        .unwrap()
        .build()
        .unwrap();
    let reasoner = Reasoner::builder()
        .profile(Profile::El)
        .build(ontology)
        .unwrap();
    let direct = get_sub_object_properties(&reasoner, "http://example.org/p", true).unwrap();
    assert_eq!(direct, vec!["http://example.org/q"]);
}

#[test]
fn classify_auto_routes_el_fixture() {
    let mut reasoner = Reasoner::builder()
        .profile(Profile::Auto)
        .build(el_chain_ontology())
        .unwrap();
    let outcome = classify(&mut reasoner).unwrap();
    match outcome {
        ClassifyOutcome::Taxonomy(t) => {
            let a = reasoner
                .ontology()
                .lookup_entity("http://example.org/A")
                .unwrap();
            let c = reasoner
                .ontology()
                .lookup_entity("http://example.org/C")
                .unwrap();
            assert!(t.is_subsumed(a, c));
        }
        other => panic!("expected taxonomy from auto on EL fixture, got {other:?}"),
    }
}

#[test]
fn is_consistent_el_detects_unsatisfiable() {
    let mut ontology = Ontology::new();
    let a = ontology
        .entity_id("http://example.org/A", EntityKind::Class)
        .expect("A");
    let nothing = ontology
        .entity_id("http://www.w3.org/2002/07/owl#Nothing", EntityKind::Class)
        .expect("Nothing");
    ontology
        .add_axiom(Axiom::SubClassOf {
            subclass: a,
            superclass: nothing,
        })
        .expect("A sub Nothing");
    let reasoner = Reasoner::builder()
        .profile(Profile::El)
        .build(ontology)
        .unwrap();
    assert!(!is_consistent(&reasoner).unwrap());
}
