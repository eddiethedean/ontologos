use ontologos_ql::{MAX_QUERY_ATOMS, MAX_QUERY_LEN, parse_conjunctive_query};

#[test]
fn rejects_query_exceeding_max_length() {
    let query = format!("Type(?x, http://ex.org/{})", "a".repeat(MAX_QUERY_LEN));
    let err = parse_conjunctive_query(&query).expect_err("length");
    assert!(err.to_string().contains("length"));
}

#[test]
fn rejects_multi_atom_queries() {
    let query = "Type(?x, http://ex.org/A) AND SubClassOf(?x, http://ex.org/B)";
    let err = parse_conjunctive_query(query).expect_err("atoms");
    assert!(err.to_string().contains(&MAX_QUERY_ATOMS.to_string()));
}
