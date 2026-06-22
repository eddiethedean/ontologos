use std::path::Path;

use ontologos_parser::load_ontology;

#[test]
fn ni_ontology_has_both_class_assertions() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_reasonertest_testniruleblockingwithunraveling.ofn",
    );
    let ont = load_ontology(&path).expect("load");
    let mut dl_class = 0;
    let mut graph_class = 0;
    for ax in ont.dl().axioms() {
        if matches!(ax, ontologos_core::DlAxiom::ClassAssertion { .. }) {
            dl_class += 1;
            eprintln!("dl: {ax:?}");
        }
    }
    for (_, ax) in ont.axioms().iter() {
        if matches!(ax, ontologos_core::Axiom::ClassAssertion { .. }) {
            graph_class += 1;
            eprintln!("graph: {ax:?}");
        }
    }
    assert_eq!(dl_class, 2, "expected two DL class assertions");
    assert_eq!(graph_class, 1, "atomic A stays on graph too");

    let consistent = ontologos_dl::is_consistent(&ont).expect("check");
    assert!(
        !consistent,
        "NI ontology should be inconsistent, got {consistent}"
    );
}
