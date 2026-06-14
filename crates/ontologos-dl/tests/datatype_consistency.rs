use ontologos_dl::{is_consistent, is_datatype_consistent};
use ontologos_parser::load_ontology;
use std::path::PathBuf;

#[test]
fn datatype_union1_is_inconsistent() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testdatatypeunion1.ofn",
    );
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
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testdatatypeunion2.ofn",
    );
    let ontology = load_ontology(&path).expect("load");
    let consistent = is_consistent(&ontology).expect("check");
    assert!(consistent, "expected sat");
}
#[test]
fn datatypes_unsat1_is_inconsistent() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testdatatypesunsat1.ofn",
    );
    let ontology = load_ontology(&path).expect("load");
    let consistent = is_consistent(&ontology).expect("check");
    assert!(!consistent, "expected unsat");
}

#[test]
fn disjoint_dps_unsat_is_inconsistent() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testdisjointdpsunsat.ofn",
    );
    assert!(!is_consistent(&load_ontology(&path).unwrap()).unwrap());
}

#[test]
fn self_inequality_is_inconsistent() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testselfinequality.ofn",
    );
    assert!(!is_consistent(&load_ontology(&path).unwrap()).unwrap());
}

#[test]
fn neg_zero2integer_is_consistent() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testnegzero2integer.ofn",
    );
    assert!(is_consistent(&load_ontology(&path).unwrap()).unwrap());
}

#[test]
fn datatype_def1_is_consistent() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testdatatypedef1.ofn",
    );
    assert!(is_consistent(&load_ontology(&path).unwrap()).unwrap());
}

#[test]
fn disjoint_dps_sat_integer_is_consistent() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testdisjointdpssatinteger.ofn",
    );
    assert!(is_consistent(&load_ontology(&path).unwrap()).unwrap());
}

#[test]
fn datatypes_unsat3_is_inconsistent() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testdatatypesunsat3.ofn",
    );
    assert!(!is_consistent(&load_ontology(&path).unwrap()).unwrap());
}

#[test]
fn rationals2_is_inconsistent() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testrationals2.ofn",
    );
    assert!(!is_consistent(&load_ontology(&path).unwrap()).unwrap());
}

#[test]
fn float_enum_inconsistent_is_inconsistent() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testfloatenuminconsistent.ofn",
    );
    assert!(!is_consistent(&load_ontology(&path).unwrap()).unwrap());
}

#[test]
fn datatype_union3_is_consistent() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testdatatypeunion3.ofn",
    );
    assert!(is_consistent(&load_ontology(&path).unwrap()).unwrap());
}

#[test]
fn datatype_union_intersection1_is_consistent() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testdatatypeunionintersection1.ofn",
    );
    assert!(is_consistent(&load_ontology(&path).unwrap()).unwrap());
}

#[test]
fn all_values_from_mixed1_is_consistent() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testallvaluesfrommixed1.ofn",
    );
    assert!(is_consistent(&load_ontology(&path).unwrap()).unwrap());
}
