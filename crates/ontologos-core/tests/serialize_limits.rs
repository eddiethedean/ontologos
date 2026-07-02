//! B-22, B-30: JSON literal limits and format_version parsing.

use ontologos_core::{Limits, Ontology};

#[test]
fn format_version_10_is_not_rejected_by_substring_guard() {
    let json = r#"{
        "format_version": 10,
        "entities": [{"iri": "http://ex/A", "kind": "Class"}],
        "axioms": []
    }"#;
    let err = Ontology::from_json_with_limits(json, Limits::default()).unwrap_err();
    assert!(
        err.to_string().contains("unsupported format_version"),
        "expected numeric version check, not substring false positive: {err}"
    );
}

#[test]
fn oversized_literal_rejected() {
    let huge = "x".repeat(2 * 1024 * 1024);
    let json = format!(
        r#"{{
        "format_version": 2,
        "entities": [
            {{"iri": "http://ex/x", "kind": "Individual"}},
            {{"iri": "http://ex/p", "kind": "DataProperty"}}
        ],
        "axioms": [{{
            "DataPropertyAssertion": {{
                "individual": "http://ex/x",
                "property": "http://ex/p",
                "value": {{"lexical": "{huge}", "datatype": "http://www.w3.org/2001/XMLSchema#string"}}
            }}
        }}]
    }}"#
    );
    let limits = Limits {
        max_literal_bytes: 1024,
        ..Limits::default()
    };
    let err = Ontology::from_json_with_limits(&json, limits).unwrap_err();
    assert!(err.to_string().contains("literal exceeds"));
}
