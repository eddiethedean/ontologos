use std::path::Path;

use ontologos_parser::load_ontology;

#[test]
fn load_minimal_rdf_xml() {
    let path = fixture("minimal_subclass.rdf");
    let ontology = load_ontology(&path).expect("rdf/xml");
    assert!(ontology.axiom_count() >= 1);
}

#[test]
fn load_minimal_ofn() {
    let path = fixture("minimal.ofn");
    let ontology = load_ontology(&path).expect("ofn");
    assert!(ontology.axiom_count() >= 1);
}

#[test]
fn load_minimal_turtle() {
    let path = fixture("minimal.ttl");
    let ontology = load_ontology(&path).expect("turtle");
    assert!(ontology.axiom_count() >= 1);
}

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}
