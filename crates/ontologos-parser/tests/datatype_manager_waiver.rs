//! Parser-level coverage for logical datatype claims (HermiT datatype-manager waiver).
//!
//! HermiT `AnyURITest` / `RDFPlainLiteralTest` / `BinaryDataTest` / `DateTimeTest` JVM
//! cases stay excluded; these tests lock OWL-facing literal and datatype-definition loads.

use std::path::Path;

use ontologos_parser::load_ontology;

#[test]
fn hermit_datatype_literal_axioms_load_without_skips() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_reasonertest_testnegativedatapropertyassertion.ofn",
    );
    let ontology = load_ontology(&path).expect("load datatype literal OFN");
    let meta = ontology.parse_meta().expect("parse meta");
    assert_eq!(
        meta.skipped_axiom_count, 0,
        "datatype literal axioms should map cleanly: {:?}",
        meta.warnings
    );
    assert!(ontology.entity_count() > 0);
}

#[test]
fn hermit_datatype_definition_entailment_premise_loads() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_reasonertest_testdatatypedefentailment.ofn",
    );
    if !path.is_file() {
        return;
    }
    let ontology = load_ontology(&path).expect("load datatype definition OFN");
    assert!(ontology.axiom_count() > 0 || ontology.entity_count() > 0);
}

#[test]
fn xsd_typed_literal_snippet_loads() {
    let ofn = r#"Prefix(:=<file:/c/test.owl#>)
Prefix(a:=<file:/c/test.owl#>)
Prefix(rdfs:=<http://www.w3.org/2000/01/rdf-schema#>)
Prefix(owl:=<http://www.w3.org/2002/07/owl#>)
Prefix(xsd:=<http://www.w3.org/2001/XMLSchema#>)
Ontology(<file:/c/test.owl#>
Declaration(DataProperty(:dp))
Declaration(NamedIndividual(:a))
DataPropertyAssertion(:dp :a "abc"^^xsd:string)
)"#;
    let dir = std::env::temp_dir().join("ontologos_datatype_waiver");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("literal.ofn");
    std::fs::write(&path, ofn).expect("write ofn");
    let ontology = load_ontology(&path).expect("load literal snippet");
    assert!(
        ontology.lookup_entity("file:/c/test.owl#a").is_some(),
        "individual :a should be declared"
    );
}

#[test]
fn data_one_of_restriction_snippet_loads() {
    let ofn = r#"Prefix(:=<file:/c/test.owl#>)
Prefix(a:=<file:/c/test.owl#>)
Prefix(rdfs:=<http://www.w3.org/2000/01/rdf-schema#>)
Prefix(owl:=<http://www.w3.org/2002/07/owl#>)
Prefix(xsd:=<http://www.w3.org/2001/XMLSchema#>)
Ontology(<file:/c/test.owl#>
Declaration(Class(:C))
Declaration(DataProperty(:dp))
SubClassOf(:C DataSomeValuesFrom(:dp DataOneOf("a"^^xsd:string "b"^^xsd:string)))
)"#;
    let dir = std::env::temp_dir().join("ontologos_datatype_waiver");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("oneof.ofn");
    std::fs::write(&path, ofn).expect("write ofn");
    let ontology = load_ontology(&path).expect("load one-of snippet");
    assert!(
        ontology.lookup_entity("file:/c/test.owl#C").is_some(),
        "class :C should be declared"
    );
}
