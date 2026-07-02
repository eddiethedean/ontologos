//! Tier-A ports from HermiT `ReasonerTest` and `OWLLinkTest` (RDFS fragment).
//!
//! Source: `HermiT/project/test/org/semanticweb/HermiT/reasoner/ReasonerTest.java`
//!         `HermiT/project/test/org/semanticweb/HermiT/reasoner/OWLLinkTest.java`

use ontologos_conformance::{
    PORT_NS, assert_direct_subproperty, assert_subproperty, assert_subsumed, assert_typed,
};
use ontologos_core::Ontology;
use ontologos_rl::rdfs::RdfsEngine;

fn iri(local: &str) -> String {
    format!("{PORT_NS}{local}")
}

fn materialize(ontology: &mut Ontology) {
    RdfsEngine::new()
        .materialize(ontology)
        .expect("materialize");
}

/// HermiT `ReasonerTest.testSubsumption1`
///
/// ```text
/// SubClassOf(:Person :Animal)
/// SubClassOf(:Student :Person)
/// SubClassOf(:Dog :Animal)
/// ```
///
/// Expect: Student ⊑ Animal; not Student ⊑ Dog; not Animal ⊑ Student.
#[test]
fn subsumption1_transitive_subclass() {
    let mut ontology = Ontology::builder()
        .class(&iri("Person"))
        .expect("Person")
        .class(&iri("Animal"))
        .expect("Animal")
        .class(&iri("Student"))
        .expect("Student")
        .class(&iri("Dog"))
        .expect("Dog")
        .subclass_of(&iri("Person"), &iri("Animal"))
        .expect("Person sub Animal")
        .subclass_of(&iri("Student"), &iri("Person"))
        .expect("Student sub Person")
        .subclass_of(&iri("Dog"), &iri("Animal"))
        .expect("Dog sub Animal")
        .build()
        .expect("build");

    materialize(&mut ontology);

    assert!(assert_subsumed(&ontology, &iri("Student"), &iri("Animal")));
    assert!(!assert_subsumed(&ontology, &iri("Animal"), &iri("Student")));
    assert!(!assert_subsumed(&ontology, &iri("Student"), &iri("Dog")));
    assert!(!assert_subsumed(&ontology, &iri("Dog"), &iri("Student")));
}

/// HermiT `OWLLinkTest.testUpdatesBuffered` — initial axiom set before ontology edits.
///
/// ```text
/// SubClassOf(:A :B)
/// SubClassOf(:B :C)
/// ```
///
/// HermiT expects hierarchy with A ⊑ B ⊑ C under `file:/c/test.owl#` (see `updateHierarchy.txt`).
#[test]
fn owllink_update_hierarchy_buffered_initial() {
    let mut ontology = Ontology::builder()
        .class(&iri("A"))
        .expect("A")
        .class(&iri("B"))
        .expect("B")
        .class(&iri("C"))
        .expect("C")
        .subclass_of(&iri("A"), &iri("B"))
        .expect("A sub B")
        .subclass_of(&iri("B"), &iri("C"))
        .expect("B sub C")
        .build()
        .expect("build");

    materialize(&mut ontology);

    assert!(assert_subsumed(&ontology, &iri("A"), &iri("C")));
    assert!(assert_subsumed(&ontology, &iri("A"), &iri("B")));
    assert!(assert_subsumed(&ontology, &iri("B"), &iri("C")));
}

/// HermiT `OWLLinkTest.testUpdatesBuffered` — axioms after flush (RDFS-relevant subset).
///
/// After removing `A ⊑ B` and adding `D ⊑ E`, `E ⊑ F`:
/// Expect D ⊑ F via transitive closure; A no longer subsumed by B.
#[test]
fn owllink_update_hierarchy_buffered_flushed() {
    let mut ontology = Ontology::builder()
        .class(&iri("B"))
        .expect("B")
        .class(&iri("C"))
        .expect("C")
        .class(&iri("D"))
        .expect("D")
        .class(&iri("E"))
        .expect("E")
        .class(&iri("F"))
        .expect("F")
        .subclass_of(&iri("B"), &iri("C"))
        .expect("B sub C")
        .subclass_of(&iri("D"), &iri("E"))
        .expect("D sub E")
        .subclass_of(&iri("E"), &iri("F"))
        .expect("E sub F")
        .build()
        .expect("build");

    materialize(&mut ontology);

    assert!(assert_subsumed(&ontology, &iri("D"), &iri("F")));
    assert!(assert_subsumed(&ontology, &iri("B"), &iri("C")));
}

/// Same RDFS expectations as buffered test — OntoLogos has no OWL API buffered reasoner in v0.3.
#[test]
fn owllink_update_hierarchy_non_buffered() {
    owllink_update_hierarchy_buffered_initial();
    owllink_update_hierarchy_buffered_flushed();
}

/// HermiT `ReasonerTest.testSubAndSuperConcepts`
#[test]
fn sub_and_super_concepts() {
    let mut ontology = Ontology::builder()
        .class(&iri("C"))
        .expect("C")
        .class(&iri("D"))
        .expect("D")
        .class(&iri("E"))
        .expect("E")
        .subclass_of(&iri("C"), &iri("D"))
        .expect("C sub D")
        .subclass_of(&iri("D"), &iri("E"))
        .expect("D sub E")
        .build()
        .expect("build");

    materialize(&mut ontology);

    assert!(assert_subsumed(&ontology, &iri("C"), &iri("D")));
    assert!(assert_subsumed(&ontology, &iri("C"), &iri("E")));
    assert!(assert_subsumed(&ontology, &iri("D"), &iri("E")));
    assert!(!assert_subsumed(&ontology, &iri("D"), &iri("C")));
    assert!(!assert_subsumed(&ontology, &iri("E"), &iri("C")));
    assert!(!assert_subsumed(&ontology, &iri("E"), &iri("D")));
}

/// HermiT `ReasonerTest.testSubAndSuperRoles`
#[test]
fn sub_and_super_roles() {
    let mut ontology = Ontology::builder()
        .object_property(&iri("r"))
        .expect("r")
        .object_property(&iri("s"))
        .expect("s")
        .object_property(&iri("t"))
        .expect("t")
        .subproperty_of(&iri("r"), &iri("s"))
        .expect("r sub s")
        .subproperty_of(&iri("s"), &iri("t"))
        .expect("s sub t")
        .build()
        .expect("build");

    materialize(&mut ontology);

    assert!(assert_subproperty(&ontology, &iri("r"), &iri("s")));
    assert!(assert_direct_subproperty(&ontology, &iri("r"), &iri("t")));
    assert!(assert_subproperty(&ontology, &iri("s"), &iri("t")));
    assert!(!assert_subproperty(&ontology, &iri("s"), &iri("r")));
    assert!(!assert_subproperty(&ontology, &iri("t"), &iri("r")));
    assert!(!assert_subproperty(&ontology, &iri("t"), &iri("s")));
}

/// Requires materialization: domain typing from property assertion (prp-dom).
#[test]
fn domain_typing_from_property_assertion() {
    let mut ontology = Ontology::builder()
        .class(&iri("Person"))
        .expect("Person")
        .individual(&iri("alice"))
        .expect("alice")
        .individual(&iri("bob"))
        .expect("bob")
        .object_property(&iri("knows"))
        .expect("knows")
        .property_domain(&iri("knows"), &iri("Person"))
        .expect("knows domain Person")
        .object_property_assertion(&iri("alice"), &iri("knows"), &iri("bob"))
        .expect("alice knows bob")
        .build()
        .expect("build");

    assert!(
        !assert_typed(&ontology, &iri("alice"), &iri("Person")),
        "alice should not be typed before materialization"
    );

    materialize(&mut ontology);

    assert!(
        assert_typed(&ontology, &iri("alice"), &iri("Person")),
        "materialization should infer alice rdf:type Person from domain"
    );
}
