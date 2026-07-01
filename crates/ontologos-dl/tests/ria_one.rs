use ontologos_alc::{TableauSeed, tableau_is_consistent_with_seed};
use ontologos_parser::load_ontology;
use std::path::PathBuf;

#[test]
fn satisfiability_with_rias11b_is_inconsistent() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_reasonertest_testsatisfiabilitywithrias11b.ofn",
    );
    let ontology = load_ontology(&path).expect("load");
    assert!(
        !tableau_is_consistent_with_seed(&ontology, &TableauSeed::default()).expect("tableau"),
        "RIA 11b fixture should be inconsistent under ALC tableau"
    );
}
