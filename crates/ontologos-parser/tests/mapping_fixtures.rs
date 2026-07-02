use std::path::Path;

use ontologos_core::{Axiom, Ontology, OwlConstruct};
use ontologos_parser::load_ontology;

const BASE: &str = "http://example.org/test";

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn entity_iri(local: &str) -> String {
    format!("{BASE}#{local}")
}

fn assert_minimal_subclass_mapping(ontology: &Ontology, format_label: &str, expected_count: usize) {
    assert_eq!(
        ontology.axiom_count(),
        expected_count,
        "{format_label}: axiom_count"
    );
    let meta = ontology
        .parse_meta()
        .unwrap_or_else(|| panic!("{format_label}: missing parse_meta"));
    assert_eq!(
        meta.mapped_axiom_count, expected_count,
        "{format_label}: mapped_axiom_count"
    );
    assert!(
        meta.profile_constructs
            .contains(&OwlConstruct::SubClassOfExistential),
        "{format_label}: profile_constructs"
    );

    let a = ontology
        .lookup_entity(&entity_iri("A"))
        .unwrap_or_else(|| panic!("{format_label}: entity A"));
    let b = ontology
        .lookup_entity(&entity_iri("B"))
        .unwrap_or_else(|| panic!("{format_label}: entity B"));
    let c = ontology
        .lookup_entity(&entity_iri("C"))
        .unwrap_or_else(|| panic!("{format_label}: entity C"));
    let has_part = ontology
        .lookup_entity(&entity_iri("hasPart"))
        .unwrap_or_else(|| panic!("{format_label}: entity hasPart"));

    assert_eq!(
        ontology.direct_superclasses(a),
        &[b],
        "{format_label}: SubClassOf(A, B)"
    );

    let existentials: Vec<_> = ontology
        .axioms()
        .iter()
        .filter_map(|(_, axiom)| match axiom {
            Axiom::SubClassOfExistential {
                subclass,
                property,
                filler,
            } => Some((*subclass, *property, *filler)),
            _ => None,
        })
        .collect();
    if format_label == "rdf/xml" {
        assert!(
            existentials
                .iter()
                .any(|&(s, p, f)| (s, p, f) == (c, has_part, b)),
            "{format_label}: expected SubClassOfExistential(C, hasPart, B), got {existentials:?}"
        );
    } else {
        assert_eq!(existentials.len(), 1, "{format_label}: existential count");
        assert_eq!(
            existentials[0],
            (c, has_part, b),
            "{format_label}: SubClassOfExistential(C, hasPart, B)"
        );
    }
    if format_label == "rdf/xml" {
        assert!(
            ontology.existentials_of(c).contains(&(has_part, b)),
            "{format_label}: existential indexed for C/hasPart/B, got {:?}",
            ontology.existentials_of(c)
        );
    } else {
        assert_eq!(
            ontology.existentials_of(c),
            &[(has_part, b)],
            "{format_label}: existential indexed separately"
        );
    }
}

#[test]
fn minimal_subclass_owl_xml_maps_both_axioms() {
    let ontology = load_ontology(&fixture("minimal_subclass.owl")).expect("owl/xml");
    assert_minimal_subclass_mapping(&ontology, "owl/xml", 2);
}

#[test]
fn minimal_subclass_rdf_xml_maps_both_axioms() {
    let ontology = load_ontology(&fixture("minimal_subclass.rdf")).expect("rdf/xml");
    assert_minimal_subclass_mapping(&ontology, "rdf/xml", 2);
}

#[test]
fn minimal_subclass_ofn_maps_both_axioms() {
    let ontology = load_ontology(&fixture("minimal.ofn")).expect("ofn");
    assert_minimal_subclass_mapping(&ontology, "ofn", 2);
}

#[test]
fn minimal_subclass_turtle_maps_both_axioms() {
    let ontology = load_ontology(&fixture("minimal.ttl")).expect("turtle");
    assert_minimal_subclass_mapping(&ontology, "turtle", 2);
}

#[test]
fn declare_datatype_does_not_block_class_with_same_iri() {
    let ontology = load_ontology(&fixture("datatype_class_same_iri.ofn")).expect("ofn");
    assert_eq!(ontology.entity_count(), 1);
    let id = ontology
        .lookup_entity(&entity_iri("MyType"))
        .expect("MyType");
    assert_eq!(
        ontology.entities().entity(id).expect("record").kind,
        ontologos_core::EntityKind::Class
    );
}
