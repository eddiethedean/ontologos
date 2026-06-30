//! HermiT `NormalizationTest` ports (engine-internal → unit tests).
//!
//! HermiT checks OWL structural normalization strings; OntoLogos smoke-tests clausify
//! on the same input axioms until full normalization parity lands.

use ontologos_alc::clausify;
use ontologos_parser::load_ontology;
use std::sync::atomic::{AtomicU64, Ordering};

const NS: &str = "file:/c/test.owl#";
static TEMP_OFN_COUNTER: AtomicU64 = AtomicU64::new(0);

fn wrap_axioms(axioms: &str) -> String {
    format!(
        "Prefix(:=<{NS}>)\n\
         Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
         Prefix(xsd:=<http://www.w3.org/2001/XMLSchema#>)\n\
         Ontology(<{NS}>\n{axioms}\n)\n"
    )
}

fn clausify_axioms(axioms: &str) -> ontologos_alc::ClauseSet {
    let id = TEMP_OFN_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "ontologos_norm_test_{}_{}.ofn",
        std::process::id(),
        id
    ));
    std::fs::write(&dir, wrap_axioms(axioms)).expect("write temp ofn");
    let mut ontology = load_ontology(&dir).expect("load");
    clausify(&mut ontology).expect("clausify")
}

/// HermiT `NormalizationTest` input axioms (from NormalizationTest.java).
const NORMALIZATION_SMOKE: &[(&str, &str)] = &[
    (
        "structural.NormalizationTest.testDataPropertiesHasValue1",
        "Declaration(Class(:Eighteen)) Declaration(DataProperty(:hasAge)) \
         SubClassOf(:Eighteen DataHasValue(:hasAge \"18\"^^xsd:integer))",
    ),
    (
        "structural.NormalizationTest.testDataPropertiesHasValue2",
        "Declaration(Class(:Eighteen)) Declaration(DataProperty(:hasAge)) \
         SubClassOf(DataHasValue(:hasAge \"18\"^^xsd:integer) :Eighteen)",
    ),
    (
        "structural.NormalizationTest.testDataPropertiesAll1",
        "Declaration(Class(:A)) Declaration(DataProperty(:dp)) \
         SubClassOf(:A DataAllValuesFrom(:dp xsd:integer))",
    ),
    (
        "structural.NormalizationTest.testDataPropertiesAll2",
        "Declaration(Class(:A)) Declaration(DataProperty(:dp)) \
         SubClassOf(DataAllValuesFrom(:dp xsd:integer) :A)",
    ),
    (
        "structural.NormalizationTest.testDataPropertiesSome1",
        "Declaration(Class(:A)) Declaration(DataProperty(:dp)) \
         SubClassOf(DataSomeValuesFrom(:dp xsd:string) :A)",
    ),
    (
        "structural.NormalizationTest.testDataPropertiesSome2",
        "Declaration(Class(:A)) Declaration(DataProperty(:dp)) \
         SubClassOf(:A DataSomeValuesFrom(:dp xsd:string))",
    ),
    (
        "structural.NormalizationTest.testDataPropertiesDataOneOf1",
        "Declaration(Class(:A)) Declaration(DataProperty(:dp)) \
         SubClassOf(:A DataAllValuesFrom(:dp DataOneOf(\"Peter\"^^xsd:string \"19\"^^xsd:integer)))",
    ),
    (
        "structural.NormalizationTest.testDataPropertiesDataOneOf2",
        "Declaration(Class(:A)) Declaration(DataProperty(:dp)) \
         SubClassOf(DataAllValuesFrom(:dp DataOneOf(\"18\"^^xsd:integer \"19\"^^xsd:integer)) :A)",
    ),
    (
        "structural.NormalizationTest.testDataPropertiesDataComplementOf1",
        "SubClassOf(:A DataAllValuesFrom(:dp DataComplementOf(DataComplementOf(DataOneOf(\"18\"^^xsd:integer \"19\"^^xsd:integer)))))",
    ),
    (
        "structural.NormalizationTest.testDataPropertiesMax1",
        "Declaration(Class(:A)) Declaration(DataProperty(:dp)) \
         SubClassOf(:A DataMaxCardinality(1 :dp xsd:string))",
    ),
    (
        "structural.NormalizationTest.testDataPropertiesMax2",
        "Declaration(Class(:A)) Declaration(DataProperty(:dp)) \
         SubClassOf(DataMaxCardinality(1 :dp xsd:string) :A)",
    ),
    (
        "structural.NormalizationTest.testDataPropertiesMax3",
        "Declaration(Class(:A)) Declaration(DataProperty(:dp)) \
         SubClassOf(:A DataMaxCardinality(5 :dp xsd:integer))",
    ),
    (
        "structural.NormalizationTest.testDataPropertiesMax4",
        "Declaration(Class(:A)) Declaration(DataProperty(:dp)) \
         SubClassOf(DataMaxCardinality(5 :dp xsd:integer) :A)",
    ),
    (
        "structural.NormalizationTest.testDataPropertiesMin1",
        "Declaration(Class(:A)) Declaration(DataProperty(:dp)) \
         SubClassOf(DataMinCardinality(1 :dp xsd:string) :A)",
    ),
    (
        "structural.NormalizationTest.testDataPropertiesMin2",
        "Declaration(Class(:A)) Declaration(DataProperty(:dp)) \
         SubClassOf(DataMinCardinality(3 :dp xsd:string) :A)",
    ),
    (
        "structural.NormalizationTest.testDataPropertiesMin3",
        "Declaration(Class(:A)) Declaration(DataProperty(:dp)) \
         SubClassOf(:A DataMinCardinality(1 :dp xsd:integer))",
    ),
    (
        "structural.NormalizationTest.testDataPropertiesMin4",
        "Declaration(Class(:A)) Declaration(DataProperty(:dp)) \
         SubClassOf(:A DataMinCardinality(3 :dp xsd:integer))",
    ),
    (
        "structural.NormalizationTest.testDataPropertiesExact1",
        "Declaration(Class(:A)) Declaration(DataProperty(:dp)) \
         SubClassOf(:A DataExactCardinality(1 :dp xsd:integer))",
    ),
    (
        "structural.NormalizationTest.testDataPropertiesExact2",
        "Declaration(Class(:A)) Declaration(DataProperty(:dp)) \
         SubClassOf(DataExactCardinality(1 :dp xsd:integer) :A)",
    ),
    (
        "structural.NormalizationTest.testDataPropertiesExact3",
        "Declaration(Class(:A)) Declaration(DataProperty(:dp)) \
         SubClassOf(DataExactCardinality(1 :dp xsd:integer) :A)",
    ),
    (
        "structural.NormalizationTest.testDataPropertiesExact4",
        "Declaration(Class(:A)) Declaration(DataProperty(:dp)) \
         SubClassOf(DataExactCardinality(3 :dp xsd:integer) :A)",
    ),
    (
        "structural.NormalizationTest.testKeys1",
        "HasKey(:C (:r) (:dp))",
    ),
    (
        "structural.NormalizationTest.testKeys2",
        "HasKey(ObjectIntersectionOf(:A :B) (:r) (:dp))",
    ),
    (
        "structural.NormalizationTest.testTopObjectPropertyInSuperPosition",
        "SubObjectPropertyOf(:A owl:topObjectProperty)",
    ),
];

#[test]
fn hermit_normalization_smoke_catalog() {
    for (id, axioms) in NORMALIZATION_SMOKE {
        let clauses = clausify_axioms(axioms);
        assert!(
            !clauses.clauses().is_empty(),
            "{id}: expected clausify output"
        );
    }
    assert_eq!(
        NORMALIZATION_SMOKE.len(),
        24,
        "HermiT NormalizationTest count"
    );
}
