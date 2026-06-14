use ontologos_dl::is_consistent;
use ontologos_parser::load_ontology;
use std::path::PathBuf;

#[test]
#[ignore = "Widmann blocking — requires complete TBox unraveling"]
fn widmann1_is_inconsistent() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_reasonertest_testwidmann1.ofn",
    );
    let ont = load_ontology(&path).expect("load");
    let consistent = is_consistent(&ont).expect("check");
    assert!(!consistent, "expected inconsistent, got {consistent}");
}

#[test]
#[ignore = "NI rule blocking with unraveling — pending tableau blocking"]
fn ni_rule_blocking_with_unraveling_is_inconsistent() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_reasonertest_testniruleblockingwithunraveling.ofn",
    );
    let ont = load_ontology(&path).expect("load");
    let consistent = is_consistent(&ont).expect("check");
    assert!(!consistent, "expected inconsistent, got {consistent}");
}
