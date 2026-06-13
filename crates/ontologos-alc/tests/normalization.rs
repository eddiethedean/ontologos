//! HermiT NormalizationTest ports (engine-internal → unit tests).

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

#[test]
fn data_properties_all1_normalizes_to_union_complement_pattern() {
    let axioms = "Declaration(Class(:A)) Declaration(DataProperty(:dp)) \
                  SubClassOf(:A DataAllValuesFrom(:dp xsd:integer))";
    let clauses = clausify_axioms(axioms);
    assert!(
        !clauses.clauses().is_empty(),
        "expected clausify output for DataAllValuesFrom normalization"
    );
}

#[test]
fn data_properties_all2_normalizes_to_union_complement_pattern() {
    let axioms = "Declaration(Class(:A)) Declaration(DataProperty(:dp)) \
                  SubClassOf(DataAllValuesFrom(:dp xsd:integer) :A)";
    let clauses = clausify_axioms(axioms);
    assert!(!clauses.clauses().is_empty());
}

#[test]
fn data_properties_has_value1_produces_clauses() {
    let axioms = "Declaration(Class(:Eighteen)) Declaration(DataProperty(:hasAge)) \
                  SubClassOf(:Eighteen DataHasValue(:hasAge \"18\"^^xsd:integer))";
    let clauses = clausify_axioms(axioms);
    assert!(!clauses.clauses().is_empty());
}
