use ontologos_alc::is_consistent as alc_consistent;
use ontologos_dl::is_consistent;
use ontologos_parser::load_ontology;
use std::path::PathBuf;

fn ax(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/axioms")
        .join(name)
}

#[test]
fn smoke_kb_consistency() {
    for (name, ofn) in [
        ("nominals3", "hermit_reasoner_reasonertest_testnominals3.ofn"),
        ("exists_self2", "hermit_reasoner_reasonertest_testexistsself2.ofn"),
    ] {
        let ont = load_ontology(&ax(ofn)).unwrap();
        let alc = alc_consistent(&ont);
        let full = is_consistent(&ont);
        eprintln!("{name}: alc={alc:?} dl={full:?}");
    }
}
