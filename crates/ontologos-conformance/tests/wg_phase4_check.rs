//! OWL WG Phase 4 regression cases (planned backlog).
use ontologos_conformance::{check_wg_case, read_wg_catalog_file};
use ontologos_dl::is_consistent;
use ontologos_parser::load_ontology;
use std::path::PathBuf;

fn wg_fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit")
        .join(rel)
}

fn assert_consistent(rel: &str, expected: bool) {
    let path = wg_fixture(rel);
    let ont = load_ontology(&path).expect("load");
    let actual = is_consistent(&ont).expect("check");
    assert_eq!(
        actual,
        expected,
        "{}: expected {expected}, got {actual}",
        path.display()
    );
}

fn case_by_suffix(suffix: &str) -> ontologos_conformance::WgCase {
    read_wg_catalog_file()
        .into_iter()
        .find(|c| c.id.contains(suffix))
        .unwrap_or_else(|| panic!("missing case with suffix {suffix}"))
}

#[test]
fn wg_thing_003_inconsistent() {
    assert_consistent("wg/TestCase-3AWebOnt-2DThing-2D003/premise.rdf", false);
}

#[test]
fn wg_pattern_disjointness_inconsistent() {
    assert_consistent(
        "wg/Inconsistent-2Dpattern-2Ddisjointness/premise.ofn",
        false,
    );
}

#[test]
fn wg_thing_003_check() {
    check_wg_case(&case_by_suffix("Thing-2D003")).expect("inconsistent");
}

#[test]
fn wg_pattern_disjointness_check() {
    check_wg_case(&case_by_suffix("pattern-2Ddisjointness")).expect("inconsistent");
}

#[test]
fn wg_punning_negative_entailment() {
    check_wg_case(&case_by_suffix("Individual-2DClass_Punning")).expect("should not entail");
}

#[test]
fn wg_keys_004_negative_entailment() {
    check_wg_case(&case_by_suffix("New-2DFeature-2DKeys-2D004")).expect("should not entail");
}

#[test]
fn wg_imports_010_entailment() {
    check_wg_case(&case_by_suffix("imports-2D010")).expect("should entail");
}

#[test]
fn wg_imports_011_entailment() {
    check_wg_case(&case_by_suffix("imports-2D011")).expect("should entail");
}

#[test]
fn wg_i46_negative_entailment() {
    check_wg_case(&case_by_suffix("I4.6-2D004")).expect("should not entail");
}

#[test]
fn wg_allvaluesfrom_002_negative_entailment() {
    check_wg_case(&case_by_suffix("allValuesFrom-2D002")).expect("should not entail");
}

#[test]
fn wg_dl_501_consistent() {
    check_wg_case(&case_by_suffix("description-2Dlogic-2D501")).expect("consistent");
}

#[test]
fn wg_dl_502_inconsistent() {
    check_wg_case(&case_by_suffix("description-2Dlogic-2D502")).expect("inconsistent");
}

#[test]
fn wg_eqclass_sym_entailment() {
    check_wg_case(&case_by_suffix("eqdis-2Deqclass-2Dsym")).expect("should entail");
}

#[test]
fn wg_equivalent_class_002_entailment() {
    check_wg_case(&case_by_suffix("equivalentClass-2D002")).expect("should entail");
}

#[test]
fn wg_complement_001_entailment() {
    check_wg_case(&case_by_suffix("complementOf-2D001")).expect("should entail");
}

#[test]
fn wg_oneof_002_entailment() {
    check_wg_case(&case_by_suffix("oneOf-2D002")).expect("should entail");
}
