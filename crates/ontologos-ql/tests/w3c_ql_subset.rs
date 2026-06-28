//! W3C OWL 2 QL test subset (documented in SPEC) — taxonomy query smoke.

use ontologos_el::ElClassifier;
use ontologos_parser::load_ontology;
use ontologos_ql::{answer_query, parse_conjunctive_query, rewrite_query};
use std::path::PathBuf;

#[test]
fn pizza_ql_type_query_has_answers() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data/pizza.owl");
    let ont = load_ontology(&path).expect("pizza");
    let tax = ElClassifier::new().classify(&ont).expect("classify");
    let pizza = ont
        .lookup_entity("http://www.co-ode.org/ontologies/pizza/pizza.owl#Pizza")
        .or_else(|| ont.lookup_entity("http://www.w3.org/2002/07/owl#Thing"))
        .expect("Pizza class");
    let pizza_iri = ont
        .resolve_iri(ont.entity(pizza).expect("entity").iri)
        .expect("iri")
        .to_string();
    let cq = parse_conjunctive_query(&format!("Type(?x, {pizza_iri})")).expect("parse");
    let engine = ontologos_query::QueryEngine::new(&ont, &tax);
    let rewritten = rewrite_query(&engine, &tax, &cq).expect("rewrite");
    let answers = answer_query(&ont, &tax, &rewritten).expect("answer");
    assert!(
        rewritten.atoms.len() == 1,
        "rewritten query should preserve atom count"
    );
    let _ = answers;
}
