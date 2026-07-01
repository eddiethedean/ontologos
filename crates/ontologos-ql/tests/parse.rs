//! OWL QL query parser tests.

use ontologos_ql::parse_conjunctive_query;

#[test]
fn parses_type_atom() {
    let cq = parse_conjunctive_query("Type(?x, http://ex.org/A)").expect("parse");
    assert_eq!(cq.atoms.len(), 1);
}

#[test]
fn rejects_multi_atom_query() {
    let err = parse_conjunctive_query(
        "Type(?x, http://ex.org/A) AND SubClassOf(?y, http://ex.org/B)",
    )
    .expect_err("parse");
    assert!(err.to_string().contains("atom count"));
}
