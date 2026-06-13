use ontologos_parser::load_ontology;
use ontologos_dl::is_consistent;
use std::path::PathBuf;

#[test]
fn datatype_union1_is_inconsistent() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testdatatypeunion1.ofn");
    let ontology = load_ontology(&path).expect("load");
    let consistent = is_consistent(&ontology).expect("check");
    assert!(!consistent, "expected unsat");
}

#[test]
fn all_values_from_integer1_is_inconsistent() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testallvaluesfrominteger1.ofn");
    let ontology = load_ontology(&path).expect("load");
    let consistent = is_consistent(&ontology).expect("check");
    assert!(!consistent, "expected unsat");
}

#[test]
fn datatype_union2_is_consistent() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testdatatypeunion2.ofn");
    let ontology = load_ontology(&path).expect("load");
    let consistent = is_consistent(&ontology).expect("check");
    assert!(consistent, "expected sat");
}
#[test]
fn datatypes_unsat1_is_inconsistent() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testdatatypesunsat1.ofn");
    let ontology = load_ontology(&path).expect("load");
    let consistent = is_consistent(&ontology).expect("check");
    assert!(!consistent, "expected unsat");
}
