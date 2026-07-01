//! W3C OWL 2 QL test subset (documented in SPEC) — taxonomy query smoke.

use ontologos_core::Ontology;
use ontologos_el::ElClassifier;
use ontologos_ql::{answer_query, parse_conjunctive_query, rewrite_query};

#[test]
fn ql_subclass_query_returns_direct_subclasses() {
    let ont = Ontology::builder()
        .class("http://ex.org/A")
        .expect("A")
        .class("http://ex.org/B")
        .expect("B")
        .subclass_of("http://ex.org/A", "http://ex.org/B")
        .expect("A sub B")
        .build()
        .expect("build");
    let tax = ElClassifier::new().classify(&ont).expect("classify");
    let cq = parse_conjunctive_query("SubClassOf(?x, http://ex.org/B)").expect("parse");
    let engine = ontologos_query::QueryEngine::new(&ont, &tax);
    let rewritten = rewrite_query(&engine, &tax, &cq).expect("rewrite");
    let answers = answer_query(&ont, &tax, &rewritten).expect("answer");
    assert_eq!(rewritten.atoms.len(), 1);
    assert_eq!(answers.len(), 1, "expected one direct subclass of B");
    let a = ont.lookup_entity("http://ex.org/A").expect("A");
    assert_eq!(answers[0].bindings, vec![("x".to_owned(), a)]);
}

#[test]
fn ql_type_query_returns_direct_subclasses_of_named_class() {
    let ont = Ontology::builder()
        .class("http://ex.org/A")
        .expect("A")
        .class("http://ex.org/B")
        .expect("B")
        .subclass_of("http://ex.org/A", "http://ex.org/B")
        .expect("A sub B")
        .build()
        .expect("build");
    let tax = ElClassifier::new().classify(&ont).expect("classify");
    let cq = parse_conjunctive_query("Type(?x, http://ex.org/B)").expect("parse");
    let engine = ontologos_query::QueryEngine::new(&ont, &tax);
    let rewritten = rewrite_query(&engine, &tax, &cq).expect("rewrite");
    let answers = answer_query(&ont, &tax, &rewritten).expect("answer");
    assert_eq!(answers.len(), 1);
    let a = ont.lookup_entity("http://ex.org/A").expect("A");
    assert_eq!(answers[0].bindings, vec![("x".to_owned(), a)]);
}
