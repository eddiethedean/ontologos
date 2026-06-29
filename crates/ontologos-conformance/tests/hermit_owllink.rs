//! Hand-written ports for HermiT `OWLLinkTest` smoke and entailment checks.
//!
//! Source: `HermiT/project/test/org/semanticweb/HermiT/reasoner/OWLLinkTest.java`
//!
//! Full `primer.owl` import is blocked by `hasAge` / `otherOnt:age` datatype punning in horned-owl;
//! these tests use a minimal OFN fragment with the disjointness axioms the Java tests exercise.

use ontologos_conformance::{hermit_test_path, vendored_hermit_test_path};
use ontologos_core::{Axiom, DlAxiom, Ontology};
use ontologos_dl::is_consistent;
use ontologos_parser::load_ontology;
use std::path::PathBuf;

const FAMILIES_NS: &str = "http://example.com/owl/families/";

fn fixture_path(name: &str) -> PathBuf {
    let rel = format!("reasoner/res/{name}");
    vendored_hermit_test_path(&rel)
        .or_else(|| hermit_test_path(&rel))
        .unwrap_or_else(|| panic!("missing OWLLink fixture {name}"))
}

fn primer_fragment_ofn() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_owllinktest_primer_fragment.ofn",
    )
}

fn families_iri(local: &str) -> String {
    format!("{FAMILIES_NS}{local}")
}

fn load_owllink_primer_fragment() -> Ontology {
    load_ontology(&primer_fragment_ofn()).expect("owllink primer fragment")
}

fn has_disjoint_object_properties(ontology: &Ontology, left: &str, right: &str) -> bool {
    let Some(left) = ontology.lookup_entity(left) else {
        return false;
    };
    let Some(right) = ontology.lookup_entity(right) else {
        return false;
    };
    ontology.dl().axioms().any(|axiom| {
        let DlAxiom::DisjointObjectProperties(props) = axiom else {
            return false;
        };
        props.contains(&left) && props.contains(&right)
    })
}

fn has_disjoint_classes(ontology: &Ontology, left: &str, right: &str) -> bool {
    let Some(left) = ontology.lookup_entity(left) else {
        return false;
    };
    let Some(right) = ontology.lookup_entity(right) else {
        return false;
    };
    ontology.axioms().iter().any(|(_, axiom)| {
        matches!(
            axiom,
            Axiom::DisjointClasses(classes)
                if classes.contains(&left) && classes.contains(&right)
        )
    })
}

/// `OWLLinkTest.testInverses` / `testSuccessiveCalls` — primer fragment is consistent.
#[test]
fn owllink_primer_smoke() {
    let ontology = load_owllink_primer_fragment();
    assert!(ontology.entity_count() > 0);
    assert!(is_consistent(&ontology).expect("consistent"));
}

/// `OWLLinkTest.testObjectProperties` — declared property does not crash consistency check.
#[test]
fn owllink_object_properties_declaration_smoke() {
    let ontology = Ontology::builder()
        .object_property(&families_iri("hasParent"))
        .expect("hasParent")
        .build()
        .expect("build");
    assert!(is_consistent(&ontology).expect("consistent"));
}

/// `OWLLinkTest.testDisjointProperties` — hasParent and hasSpouse are disjoint in primer.
#[test]
fn owllink_disjoint_properties_has_parent_spouse() {
    let ontology = load_owllink_primer_fragment();
    assert!(has_disjoint_object_properties(
        &ontology,
        &families_iri("hasParent"),
        &families_iri("hasSpouse"),
    ));
}

/// `OWLLinkTest.testDisjointClasses` — Father is disjoint with Mother in primer.
#[test]
fn owllink_disjoint_classes_father_mother() {
    let ontology = load_owllink_primer_fragment();
    assert!(has_disjoint_classes(
        &ontology,
        &families_iri("Father"),
        &families_iri("Mother"),
    ));
}

#[test]
fn owllink_vendored_fixtures_present() {
    for name in ["primer.owl", "families.owl"] {
        assert!(
            fixture_path(name).is_file(),
            "vendored OWLLink fixture {name} should exist for future full-corpus work"
        );
    }
}
