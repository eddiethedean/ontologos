//! W3C OWL 2 QL test subset (documented in SPEC) — taxonomy query smoke.

use ontologos_core::{Axiom, EntityKind, Ontology};
use ontologos_el::ElClassifier;
use ontologos_ql::{
    OWL_NOTHING_IRI, QueryAtom, answer_query, parse_conjunctive_query, rewrite_query,
};

fn unsatisfiable_class_with_subclass() -> Ontology {
    let mut ontology = Ontology::new();
    let a = ontology
        .entity_id("http://ex.org/A", EntityKind::Class)
        .expect("A");
    let b = ontology
        .entity_id("http://ex.org/B", EntityKind::Class)
        .expect("B");
    let nothing = ontology
        .entity_id("http://www.w3.org/2002/07/owl#Nothing", EntityKind::Class)
        .expect("Nothing");
    ontology
        .add_axiom(Axiom::SubClassOf {
            subclass: a,
            superclass: nothing,
        })
        .expect("A sub Nothing");
    ontology
        .add_axiom(Axiom::SubClassOf {
            subclass: b,
            superclass: a,
        })
        .expect("B sub A");
    ontology
}

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
    let engine = ontologos_ql::TaxonomyHierarchy::new(&ont, &tax);
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
    let engine = ontologos_ql::TaxonomyHierarchy::new(&ont, &tax);
    let rewritten = rewrite_query(&engine, &tax, &cq).expect("rewrite");
    let answers = answer_query(&ont, &tax, &rewritten).expect("answer");
    assert_eq!(answers.len(), 1);
    let a = ont.lookup_entity("http://ex.org/A").expect("A");
    assert_eq!(answers[0].bindings, vec![("x".to_owned(), a)]);
}

#[test]
fn ql_type_query_over_unsatisfiable_class_returns_empty_after_rewrite() {
    let ont = unsatisfiable_class_with_subclass();
    let tax = ElClassifier::new().classify(&ont).expect("classify");
    assert!(
        tax.unsatisfiable
            .contains(&ont.lookup_entity("http://ex.org/A").expect("A")),
        "A should be unsatisfiable"
    );
    let cq = parse_conjunctive_query("Type(?x, http://ex.org/A)").expect("parse");
    let engine = ontologos_ql::TaxonomyHierarchy::new(&ont, &tax);
    let rewritten = rewrite_query(&engine, &tax, &cq).expect("rewrite");
    assert_eq!(
        rewritten.atoms,
        vec![QueryAtom::Type {
            var: "x".to_owned(),
            class: OWL_NOTHING_IRI.to_owned(),
        }]
    );
    let answers = answer_query(&ont, &tax, &rewritten).expect("answer");
    assert!(
        answers.is_empty(),
        "rewritten query over unsatisfiable class should have no bindings"
    );
}
