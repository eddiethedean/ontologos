use ontologos_parser::{load_ontology_from_bytes, load_ontology_from_bytes_lenient};

#[test]
fn load_functional_bytes_strict() {
    let ofn = br#"Prefix(:=<http://example.org/>)
Ontology(<http://example.org/o>
  Declaration(Class(:A))
  Declaration(Class(:B))
  SubClassOf(:A :B)
)"#;
    let ontology = load_ontology_from_bytes(ofn).expect("parse");
    assert!(ontology.axiom_count() >= 1);
}

#[test]
fn load_turtle_bytes_strict() {
    let ttl = br#"@prefix : <http://example.org/> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
:A a owl:Class .
:B a owl:Class .
:A rdfs:subClassOf :B .
"#;
    let ontology = load_ontology_from_bytes(ttl).expect("parse turtle");
    assert!(ontology.axiom_count() >= 1);
}

#[test]
fn load_lenient_allows_skipped_axioms() {
    let ofn = br#"Prefix(:=<http://example.org/>)
Ontology(<http://example.org/o>
  Declaration(Class(:A))
  SubClassOf(:A :B)
)"#;
    let ontology = load_ontology_from_bytes_lenient(ofn).expect("lenient parse");
    assert!(ontology.axiom_count() >= 1);
}
