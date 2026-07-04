//! Integration tests for shared JS binding logic.

use ontologos_js::{JsOntology, JsOntologyBuilder, JsReasoner, parse_profile};
use serde_json::{Value, json};

#[test]
fn builder_classify_el() {
    let mut builder = JsOntologyBuilder::new();
    builder.add_class("http://example.org/Pizza").unwrap();
    builder.add_class("http://example.org/Food").unwrap();
    builder
        .subclass_of("http://example.org/Pizza", "http://example.org/Food")
        .unwrap();
    let ontology = builder.build().unwrap();

    let mut reasoner = JsReasoner::from_ontology(&ontology, Some("el"), false, None).unwrap();
    let report = reasoner.classify().unwrap();
    assert_eq!(report["status"], "classified");
    assert!(report["subsumption_count"].as_u64().unwrap() >= 1);
}

#[test]
fn load_from_bytes_functional_strict() {
    let ofn = r#"Prefix(:=<http://example.org/>)
Ontology(<http://example.org/o>
  Declaration(Class(:A))
  Declaration(Class(:B))
  SubClassOf(:A :B)
)"#;
    let ontology = JsOntology::load_bytes(ofn.as_bytes()).unwrap();
    assert!(ontology.axiom_count().unwrap() >= 1);
}

#[test]
fn load_from_bytes_lenient() {
    let ofn = r#"Prefix(:=<http://example.org/>)
Ontology(<http://example.org/o>
  Declaration(Class(:A))
  SubClassOf(:A :B)
)"#;
    let ontology = JsOntology::load_bytes_lenient(ofn.as_bytes()).unwrap();
    assert!(ontology.axiom_count().unwrap() >= 1);
}

#[test]
fn check_consistency() {
    let mut builder = JsOntologyBuilder::new();
    builder.add_class("http://example.org/A").unwrap();
    let ontology = builder.build().unwrap();
    let mut reasoner = JsReasoner::from_ontology(&ontology, Some("el"), false, None).unwrap();
    let result = reasoner.check_consistency().unwrap();
    assert_eq!(result, json!({"consistent": true, "complete": true}));
}

#[test]
fn parse_profile_aliases() {
    assert!(parse_profile(Some("dl-preview")).is_ok());
    assert!(matches!(
        parse_profile(Some("swrl")).unwrap(),
        ontologos_core::Profile::Swrl
    ));
}

#[test]
fn shared_ontology_mutation_sync() {
    let mut builder = JsOntologyBuilder::new();
    builder.add_class("http://example.org/A").unwrap();
    builder.add_class("http://example.org/B").unwrap();
    let ontology = builder.build().unwrap();
    assert_eq!(ontology.axiom_count().unwrap(), 0);

    let mut reasoner = JsReasoner::from_ontology(&ontology, Some("el"), false, None).unwrap();
    reasoner
        .add_subclass_of("http://example.org/A", "http://example.org/B")
        .unwrap();
    assert_eq!(ontology.axiom_count().unwrap(), 1);
}

#[test]
fn is_entailed_subclass_chain() {
    let mut builder = JsOntologyBuilder::new();
    builder.add_class("http://example.org/A").unwrap();
    builder.add_class("http://example.org/B").unwrap();
    builder.add_class("http://example.org/C").unwrap();
    builder
        .subclass_of("http://example.org/A", "http://example.org/B")
        .unwrap();
    builder
        .subclass_of("http://example.org/B", "http://example.org/C")
        .unwrap();
    let ontology = builder.build().unwrap();
    let mut reasoner = JsReasoner::from_ontology(&ontology, Some("el"), false, None).unwrap();
    assert!(reasoner
        .is_entailed(
            Some("http://example.org/A"),
            Some("http://example.org/C"),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap());
}

#[test]
fn query_after_classify() {
    let mut builder = JsOntologyBuilder::new();
    builder.add_class("http://example.org/A").unwrap();
    builder.add_class("http://example.org/B").unwrap();
    builder
        .subclass_of("http://example.org/A", "http://example.org/B")
        .unwrap();
    let ontology = builder.build().unwrap();
    let mut reasoner = JsReasoner::from_ontology(&ontology, Some("el"), false, None).unwrap();
    let answers = reasoner.query("Type(?x, http://example.org/B)").unwrap();
    let arr = answers.as_array().unwrap();
    assert!(!arr.is_empty());
}

#[test]
fn from_dict_size_limit() {
    let mut obj = serde_json::Map::new();
    let entities: Vec<Value> = (0..2_000_000)
        .map(|i| Value::String(format!("http://example.org/x{i}")))
        .collect();
    obj.insert("entities".to_owned(), Value::Array(entities));
    let huge = Value::Object(obj);
    match JsOntology::from_dict(&huge) {
        Err(err) => assert!(matches!(err, ontologos_js::JsError::ResourceLimit(_))),
        Ok(_) => panic!("expected ResourceLimit error"),
    }
}

#[test]
fn incremental_classify_matches_full() {
    let mut builder = JsOntologyBuilder::new();
    for i in ["A", "B", "C", "D"] {
        builder
            .add_class(&format!("http://example.org/{i}"))
            .unwrap();
    }
    builder
        .subclass_of("http://example.org/A", "http://example.org/B")
        .unwrap();
    builder
        .subclass_of("http://example.org/B", "http://example.org/C")
        .unwrap();
    let ontology = builder.build().unwrap();

    let mut full = JsReasoner::from_ontology(&ontology, Some("el"), false, None).unwrap();
    let full_result = full.classify().unwrap();

    let mut incr = JsReasoner::from_ontology(&ontology, Some("el"), true, None).unwrap();
    let incr_result = incr.classify().unwrap();

    assert_eq!(
        full_result["subsumption_count"],
        incr_result["subsumption_count"]
    );
}

#[test]
fn parse_meta_in_classify_after_bytes_load() {
    let ofn = r#"Prefix(:=<http://example.org/>)
Ontology(<http://example.org/o>
  Declaration(Class(:A))
  Declaration(Class(:B))
  SubClassOf(:A :B)
)"#;
    let ontology = JsOntology::load_bytes(ofn.as_bytes()).unwrap();
    let mut reasoner = JsReasoner::from_ontology(&ontology, Some("el"), false, None).unwrap();
    let report = reasoner.classify().unwrap();
    // Clean loads omit parse_meta from JSON; property still accessible.
    assert!(reasoner.parse_meta().is_ok());
    assert_eq!(report["status"], "classified");
}
