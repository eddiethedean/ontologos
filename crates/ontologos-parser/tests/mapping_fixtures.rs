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

fn assert_minimal_subclass_mapping(ontology: &Ontology, format_label: &str) {
    assert_eq!(ontology.axiom_count(), 2, "{format_label}: axiom_count");
    let meta = ontology
        .parse_meta()
        .unwrap_or_else(|| panic!("{format_label}: missing parse_meta"));
    assert_eq!(
        meta.mapped_axiom_count, 2,
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
    assert_eq!(existentials.len(), 1, "{format_label}: existential count");
    assert_eq!(
        existentials[0],
        (c, has_part, b),
        "{format_label}: SubClassOfExistential(C, hasPart, B)"
    );
    assert_eq!(
        ontology.existentials_of(c),
        &[(has_part, b)],
        "{format_label}: existential indexed separately"
    );
}

#[test]
fn minimal_subclass_owl_xml_maps_both_axioms() {
    let ontology = load_ontology(&fixture("minimal_subclass.owl")).expect("owl/xml");
    assert_minimal_subclass_mapping(&ontology, "owl/xml");
}

#[test]
fn minimal_subclass_rdf_xml_maps_both_axioms() {
    let ontology = load_ontology(&fixture("minimal_subclass.rdf")).expect("rdf/xml");
    assert_minimal_subclass_mapping(&ontology, "rdf/xml");
}

#[test]
fn minimal_subclass_ofn_maps_both_axioms() {
    let ontology = load_ontology(&fixture("minimal.ofn")).expect("ofn");
    assert_minimal_subclass_mapping(&ontology, "ofn");
}

#[test]
fn minimal_subclass_turtle_maps_both_axioms() {
    let ontology = load_ontology(&fixture("minimal.ttl")).expect("turtle");
    assert_minimal_subclass_mapping(&ontology, "turtle");
}
