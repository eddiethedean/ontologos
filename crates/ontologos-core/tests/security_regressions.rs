use ontologos_core::{Error, Limits, Ontology};

#[test]
fn rejects_format_version_1_snapshot() {
    let json = r#"{
        "format_version": 1,
        "iris": ["http://example.org/A", "http://example.org/B"],
        "entities": [
            {"iri_index": 2, "kind": "Class"},
            {"iri_index": 1, "kind": "Class"}
        ],
        "axioms": [
            {"SubClassOf": {"subclass": 0, "superclass": 1}}
        ]
    }"#;
    let err = Ontology::from_json(json).expect_err("v1");
    assert!(matches!(err, Error::Serialization(_)));
}

#[test]
fn format_v2_entity_order_does_not_corrupt_taxonomy() {
    let json = r#"{
        "format_version": 2,
        "entities": [
            {"iri": "http://example.org/B", "kind": "Class"},
            {"iri": "http://example.org/A", "kind": "Class"}
        ],
        "axioms": [
            {"SubClassOf": {"subclass": "http://example.org/A", "superclass": "http://example.org/B"}}
        ]
    }"#;
    let ontology = Ontology::from_json(json).expect("load");
    let a = ontology.lookup_entity("http://example.org/A").expect("A");
    let b = ontology.lookup_entity("http://example.org/B").expect("B");
    assert_eq!(ontology.direct_superclasses(a), &[b]);
}

#[test]
fn rejects_javascript_iri() {
    let json = r#"{
        "format_version": 2,
        "entities": [{"iri": "javascript:alert(1)", "kind": "Class"}],
        "axioms": []
    }"#;
    let err = Ontology::from_json(json).expect_err("js");
    assert!(matches!(err, Error::InvalidIri(_)));
}

#[test]
fn duplicate_axioms_are_deduped() {
    let json = r#"{
        "format_version": 2,
        "entities": [
            {"iri": "http://example.org/A", "kind": "Class"},
            {"iri": "http://example.org/B", "kind": "Class"}
        ],
        "axioms": [
            {"SubClassOf": {"subclass": "http://example.org/A", "superclass": "http://example.org/B"}},
            {"SubClassOf": {"subclass": "http://example.org/A", "superclass": "http://example.org/B"}}
        ]
    }"#;
    let ontology = Ontology::from_json(json).expect("load");
    assert_eq!(ontology.axiom_count(), 1);
    let a = ontology.lookup_entity("http://example.org/A").expect("A");
    assert_eq!(ontology.direct_superclasses(a).len(), 1);
}

#[test]
fn rejects_oversized_json_input() {
    let limits = Limits {
        max_json_bytes: 32,
        ..Limits::default()
    };
    let json = r#"{"format_version":2,"entities":[],"axioms":[]}"#;
    let err = Ontology::from_json_with_limits(json, limits).expect_err("size");
    assert!(matches!(err, Error::Serialization(_)));
}
