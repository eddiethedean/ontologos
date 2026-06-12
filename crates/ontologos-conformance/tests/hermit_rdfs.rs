//! Tier-A ports from HermiT `ReasonerTest` and `OWLLinkTest` (RDFS fragment).
//!
//! Source: `HermiT/project/test/org/semanticweb/HermiT/reasoner/ReasonerTest.java`
//!         `HermiT/project/test/org/semanticweb/HermiT/reasoner/OWLLinkTest.java`

use ontologos_conformance::{assert_subsumed, PORT_NS};
use ontologos_core::Ontology;
use ontologos_rdfs::RdfsEngine;

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
