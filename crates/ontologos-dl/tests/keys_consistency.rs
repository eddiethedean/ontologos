use ontologos_dl::is_consistent;
use ontologos_parser::load_ontology;
use std::path::PathBuf;

fn hermit_axiom(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit")
        .join(name)
}

#[test]
fn keys1_same_key_with_different_individuals_is_inconsistent() {
    let path = hermit_axiom("axioms/hermit_reasoner_reasonertest_testkeys1.ofn");
    let ontology = load_ontology(&path).expect("load");
    assert!(!is_consistent(&ontology).expect("check"));
}

#[test]
fn non_unary_keys_same_key_with_different_individuals_is_inconsistent() {
    let path = hermit_axiom("axioms/hermit_reasoner_reasonertest_testnonunarykeys.ofn");
    let ontology = load_ontology(&path).expect("load");
    assert!(!is_consistent(&ontology).expect("check"));
}

#[test]
fn non_unary_keys2_allows_distinct_key_values() {
    let path = hermit_axiom("axioms/hermit_reasoner_reasonertest_testnonunarykeys2.ofn");
    let ontology = load_ontology(&path).expect("load");
    assert!(is_consistent(&ontology).expect("check"));
}

#[test]
fn keys3_allows_multiple_individuals_with_same_name_key() {
    let path = hermit_axiom("axioms/hermit_reasoner_reasonertest_testkeys3.ofn");
    let ontology = load_ontology(&path).expect("load");
    assert!(is_consistent(&ontology).expect("check"));
}
