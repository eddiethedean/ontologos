use ontologos_dl::is_consistent;
use ontologos_parser::load_ontology;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct CatalogCase {
    java_method: String,
    axiom_ofn: Option<String>,
    consistent: Option<bool>,
}

fn hermit_axiom(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit")
        .join(name)
}

fn ria_cases() -> Vec<(String, bool)> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/catalog/cases.json");
    let raw = std::fs::read_to_string(path).expect("read catalog");
    let cases: Vec<CatalogCase> = serde_json::from_str(&raw).expect("parse catalog");
    cases
        .into_iter()
        .filter(|c| {
            c.java_method.starts_with("testSatisfiabilityWithRIAs")
                && c.java_method != "testSatisfiabilityWithRIAs14"
                && c.java_method != "testSatisfiabilityWithRIAs11b"
        })
        .map(|c| {
            let ofn = c.axiom_ofn.expect("ofn");
            let expected = c.consistent.expect("consistent");
            (ofn, expected)
        })
        .collect()
}

#[test]
fn inverse_and_chain_is_inconsistent() {
    let path = hermit_axiom("axioms/hermit_reasoner_riatest_testinverseandchain.ofn");
    let ontology = load_ontology(&path).expect("load");
    let consistent = is_consistent(&ontology).expect("check");
    assert!(!consistent, "expected inconsistent");
}

#[test]
fn satisfiability_with_rias_catalog() {
    for (ofn, expected) in ria_cases() {
        let path = hermit_axiom(&ofn);
        let ontology = load_ontology(&path).unwrap_or_else(|e| panic!("load {ofn}: {e}"));
        let consistent = is_consistent(&ontology).unwrap_or_else(|e| panic!("check {ofn}: {e}"));
        assert_eq!(
            consistent, expected,
            "{ofn}: expected consistent={expected}, got {consistent}"
        );
    }
}
