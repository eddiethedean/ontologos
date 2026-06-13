//! OWL QL query parser tests.

use ontologos_ql::parse_conjunctive_query;

#[test]
fn parses_type_atom() {
    let cq = parse_conjunctive_query("Type(?x, http://ex.org/A)").expect("parse");
    assert_eq!(cq.atoms.len(), 1);
}

#[test]
fn parses_conjunctive_query() {
    let cq =
        parse_conjunctive_query("Type(?x, http://ex.org/A) AND SubClassOf(?y, http://ex.org/B)")
            .expect("parse");
    assert_eq!(cq.atoms.len(), 2);
}
