use ontologos_dl::{is_consistent, is_datatype_consistent};
use ontologos_parser::load_ontology;
use std::path::PathBuf;

fn ofn(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_{name}.ofn"))
}

fn check(name: &str, expected: bool) {
    let ont = load_ontology(&ofn(name)).expect("load");
    let dt = is_datatype_consistent(&ont);
    let full = is_consistent(&ont).expect("check");
    assert_eq!(
        full, expected,
        "{name}: expected full={expected}, got full={full}, datatype={dt}"
    );
}

#[test]
fn lang_abbreviation_is_consistent() {
    check("testlangabbreviation", true);
}

#[test]
fn rationals3_is_consistent() {
    check("testrationals3", true);
}

#[test]
fn datatype_def6_is_consistent() {
    check("testdatatypedef6", true);
}

#[test]
fn all_values_from_different_types2_is_inconsistent() {
    check("testallvaluesfromdifferenttypes2", false);
}

#[test]
fn datatype_union_intersection2_is_inconsistent() {
    check("testdatatypeunionintersection2", false);
}

#[test]
fn datatype_union_intersection3_is_inconsistent() {
    check("testdatatypeunionintersection3", false);
}

#[test]
fn datetime2_is_inconsistent() {
    check("testdatetime2", false);
}

#[test]
fn nominals_and_datatypes_from_alan_is_consistent() {
    check("testnominalsanddatatypesfromalan", true);
}

#[test]
fn datetime1_is_consistent() {
    check("testdatetime1", true);
}

#[test]
fn datatypes_unsat4_is_inconsistent() {
    check("testdatatypesunsat4", false);
}

#[test]
fn float_zeros_is_inconsistent() {
    check("testfloatzeros", false);
}

#[test]
fn different_oneofs_is_inconsistent() {
    check("testdifferentoneofs", false);
}

#[test]
fn disjoint_dps_unsat_strings_is_inconsistent() {
    check("testdisjointdpsunsatstrings", false);
}
