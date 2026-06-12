//! HermiT `ClassificationTest` Tier-B ports (require local `HermiT/` tree).

use ontologos_conformance::{assert_subsumed, hermit_available, hermit_test_path};
use ontologos_el::ElClassifier;
use ontologos_parser::load_ontology;
use std::collections::HashSet;
use std::fs;

fn load_hermit_fixture(relative: &str) -> Option<ontologos_core::Ontology> {
    let path = hermit_test_path(relative)?;
    load_ontology(&path).ok()
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
#[ignore = "requires HermiT source tree at HermiT/ or ONTOLOGOS_HERMIT_ROOT"]
fn hermit_classification_pizza_taxonomy() {
    if !hermit_available() {
        return;
    }
    let ontology = load_hermit_fixture("reasoner/res/pizza.xml").expect("pizza fixture");
    let control_path = hermit_test_path("reasoner/res/pizza.xml.txt").expect("control");
    let golden_text = fs::read_to_string(control_path).expect("read golden");
    let golden = parse_hermit_hierarchy_txt(&golden_text);

    let taxonomy = ElClassifier::new().classify(&ontology).expect("classify");
    assert_hierarchies(&ontology, &taxonomy, &golden);
}

#[test]
#[ignore = "requires HermiT source tree at HermiT/ or ONTOLOGOS_HERMIT_ROOT"]
fn hermit_classification_wine_taxonomy() {
    if !hermit_available() {
        return;
    }
    let ontology = load_hermit_fixture("reasoner/res/wine.xml").expect("wine fixture");
    let control_path = hermit_test_path("reasoner/res/wine.xml.txt").expect("control");
    let golden_text = fs::read_to_string(control_path).expect("read golden");
    let golden = parse_hermit_hierarchy_txt(&golden_text);

    let taxonomy = ElClassifier::new().classify(&ontology).expect("classify");
    assert_hierarchies(&ontology, &taxonomy, &golden);
}

#[test]
fn hermit_el_tests_skipped_without_fixture_tree() {
    if hermit_available() {
        return;
    }
    assert!(!hermit_available());
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
