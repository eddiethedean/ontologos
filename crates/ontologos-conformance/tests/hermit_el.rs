//! HermiT `ClassificationTest` ports (vendored fixtures + optional local `HermiT/` tree).

use ontologos_conformance::{
    assert_subsumed, classification_fixture_path, classification_fixtures_available,
};
use ontologos_el::ElClassifier;
use ontologos_parser::load_ontology;
use std::collections::HashSet;
use std::fs;

fn load_classification_fixture(relative: &str) -> ontologos_core::Ontology {
    let path = classification_fixture_path(relative)
        .unwrap_or_else(|| panic!("missing classification fixture: {relative}"));
    load_ontology(&path).expect("load fixture")
}

fn parse_hermit_hierarchy_txt(text: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((sub, sup)) = line.split_once(" SubClassOf ") {
            pairs.push((sub.trim().to_owned(), sup.trim().to_owned()));
        }
    }
    pairs
}

fn assert_hierarchies(
    ontology: &ontologos_core::Ontology,
    taxonomy: &ontologos_core::Taxonomy,
    golden_pairs: &[(String, String)],
) {
    for (sub, sup) in golden_pairs {
        let sub_id = ontology
            .lookup_entity(sub)
            .unwrap_or_else(|| panic!("missing subclass entity {sub}"));
        let sup_id = ontology
            .lookup_entity(sup)
            .unwrap_or_else(|| panic!("missing superclass entity {sup}"));
        assert!(
            taxonomy.is_subsumed(sub_id, sup_id) || assert_subsumed(ontology, sub, sup),
            "expected {sub} ⊑ {sup}"
        );
    }
}

#[test]
fn hermit_classification_pizza_taxonomy() {
    assert!(
        classification_fixtures_available(),
        "missing vendored pizza fixtures; run ./benchmarks/scripts/download.sh"
    );
    let ontology = load_classification_fixture("reasoner/res/pizza.xml");
    let control_path =
        classification_fixture_path("reasoner/res/pizza.xml.txt").expect("pizza control");
    let golden_text = fs::read_to_string(control_path).expect("read golden");
    let golden = parse_hermit_hierarchy_txt(&golden_text);

    let taxonomy = ElClassifier::new().classify(&ontology).expect("classify");
    assert_hierarchies(&ontology, &taxonomy, &golden);
}

#[test]
#[ignore = "horned-owl panics on legacy wine.xml duplicate rdf:ID until parser handles the error"]
fn hermit_classification_wine_taxonomy() {
    assert!(
        classification_fixtures_available(),
        "missing vendored classification fixtures; run ./benchmarks/scripts/download.sh"
    );
    let ontology = load_classification_fixture("reasoner/res/wine.xml");
    let control_path =
        classification_fixture_path("reasoner/res/wine.xml.txt").expect("wine control");
    let golden_text = fs::read_to_string(control_path).expect("read golden");
    let golden = parse_hermit_hierarchy_txt(&golden_text);

    let taxonomy = ElClassifier::new().classify(&ontology).expect("classify");
    assert_hierarchies(&ontology, &taxonomy, &golden);
}

#[test]
fn parse_hermit_hierarchy_format() {
    let sample = "http://ex.org/A SubClassOf http://ex.org/B\n";
    let pairs = parse_hermit_hierarchy_txt(sample);
    assert_eq!(
        pairs,
        vec![("http://ex.org/A".to_owned(), "http://ex.org/B".to_owned())]
    );
    let set: HashSet<_> = pairs.into_iter().collect();
    assert_eq!(set.len(), 1);
}
