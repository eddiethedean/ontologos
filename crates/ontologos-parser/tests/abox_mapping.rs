use std::path::Path;

use ontologos_core::{Axiom, Ontology};
use ontologos_parser::load_ontology;

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn entity_iri(local: &str) -> String {
    format!("http://example.org/test#{local}")
}

#[test]
fn maps_class_assertion() {
    let ontology = load_ontology(&fixture("abox_class_assertion.ttl")).expect("load");
    let alice = ontology.lookup_entity(&entity_iri("alice")).expect("alice");
    let person = ontology
        .lookup_entity(&entity_iri("Person"))
        .expect("Person");
    assert_eq!(ontology.classes_of(alice), &[person]);
    assert!(ontology.axioms().iter().any(|(_, axiom)| matches!(
        axiom,
        Axiom::ClassAssertion {
            individual,
            class,
        } if *individual == alice && *class == person
    )));
}

#[test]
fn maps_object_property_assertion() {
    let ontology = load_ontology(&fixture("abox_property_assertion.ttl")).expect("load");
    let alice = ontology.lookup_entity(&entity_iri("alice")).expect("alice");
    let bob = ontology.lookup_entity(&entity_iri("bob")).expect("bob");
    let knows = ontology.lookup_entity(&entity_iri("knows")).expect("knows");
    assert_eq!(ontology.object_assertions_of(alice), &[(knows, bob)]);
}

#[test]
fn maps_same_and_different_individuals() {
    let ontology = Ontology::builder()
        .individual(&entity_iri("a"))
        .expect("a")
        .individual(&entity_iri("b"))
        .expect("b")
        .individual(&entity_iri("c"))
        .expect("c")
        .same_individual(&[&entity_iri("a"), &entity_iri("b")])
        .expect("same")
        .different_individuals(&[&entity_iri("a"), &entity_iri("c")])
        .expect("diff")
        .build()
        .expect("build");
    let a = ontology.lookup_entity(&entity_iri("a")).expect("a");
    let b = ontology.lookup_entity(&entity_iri("b")).expect("b");
    let c = ontology.lookup_entity(&entity_iri("c")).expect("c");
    assert!(ontology.same_as(a).expect("same a").contains(&b));
    assert!(ontology.different_from(a).expect("diff a").contains(&c));
}
