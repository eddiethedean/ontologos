use ontologos_alc::{tableau_is_consistent_with_seed, TableauSeed};
use ontologos_parser::load_ontology;
use std::path::PathBuf;
#[test]
fn tableau() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/axioms/hermit_reasoner_reasonertest_testsatisfiabilitywithrias11b.ofn");
    let ontology = load_ontology(&path).expect("load");
    let _ = tableau_is_consistent_with_seed(&ontology, &TableauSeed::default()).expect("t");
}
