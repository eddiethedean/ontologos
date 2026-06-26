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

#[test]
fn wg_eqclass_trans_entailment() {
    check_wg_case(&case_by_suffix("eqdis-2Deqclass-2Dtrans")).expect("should entail");
}

#[test]
fn wg_disjoint_classes_001_entailment() {
    check_wg_case(&case_by_suffix("DisjointClasses-2D001")).expect("should entail");
}

#[test]
fn wg_disjoint_classes_003_entailment() {
    check_wg_case(&case_by_suffix("DisjointClasses-2D003")).expect("should entail");
}

#[test]
fn wg_i5_8_005_negative_entailment() {
    check_wg_case(&case_by_suffix("I5.8-2D005")).expect("should not entail");
}

#[test]
fn wg_cardinality_001_entailment() {
    check_wg_case(&case_by_suffix("cardinality-2D001")).expect("should entail");
}

#[test]
fn wg_cardinality_002_entailment() {
    check_wg_case(&case_by_suffix("cardinality-2D002")).expect("should entail");
}

#[test]
fn wg_cardinality_003_entailment() {
    check_wg_case(&case_by_suffix("cardinality-2D003")).expect("should entail");
}

#[test]
fn wg_rational_003_consistent() {
    assert_consistent("wg/New-2DFeature-2DRational-2D003/premise.rdf", true);
}

#[test]
fn wg_thing_004_consistent() {
    assert_consistent("wg/TestCase-3AWebOnt-2DThing-2D004/premise.rdf", true);
}

#[test]
fn wg_dl_005_consistent() {
    assert_consistent("wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D005/premise.rdf", true);
}

#[test]
fn wg_dl_601_inconsistent() {
    assert_consistent("wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D601/premise.rdf", false);
}

#[test]
fn wg_rational_002_inconsistent() {
    assert_consistent("wg/New-2DFeature-2DRational-2D002/premise.rdf", false);
}

#[test]
fn wg_thing_005_inconsistent() {
    assert_consistent("wg/TestCase-3AWebOnt-2DThing-2D005/premise.rdf", false);
}

#[test]
fn wg_restriction_006_entailment() {
    check_wg_case(&case_by_suffix("Restriction-2D006")).expect("should entail");
}

#[test]
fn wg_dataqcr_001_entailment() {
    check_wg_case(&case_by_suffix("DataQCR-2D001")).expect("should entail");
}

#[test]
fn wg_objectqcr_001_entailment() {
    check_wg_case(&case_by_suffix("ObjectQCR-2D001")).expect("should entail");
}

#[test]
fn wg_one_equals_two_inconsistent() {
    assert_consistent("wg/One_equals_two/premise.rdf", false);
}

#[test]
fn wg_dl650_inconsistent() {
    assert_consistent("wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D650/premise.rdf", false);
}

#[test]
fn wg_dl910_inconsistent() {
    assert_consistent("wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D910/premise.rdf", false);
}

#[test]
fn wg_consistent_but_all_unsat_entailment() {
    check_wg_case(&case_by_suffix("Consistent-2Dbut-2Dall-2Dunsat")).expect("should entail");
}

#[test]
fn wg_disjoint_union_001_entailment() {
    check_wg_case(&case_by_suffix("New-2DFeature-2DDisjointUnion-2D001")).expect("should entail");
}

#[test]
fn wg_self_restriction_002_entailment() {
    check_wg_case(&case_by_suffix("New-2DFeature-2DSelfRestriction-2D002")).expect("should entail");
}

#[test]
fn wg_functional_property_004_entailment() {
    check_wg_case(&case_by_suffix("FunctionalProperty-2D004")).expect("should entail");
}

#[test]
fn wg_i4_5_001_entailment() {
    check_wg_case(&case_by_suffix("I4.5-2D001")).expect("should entail");
}

#[test]
fn wg_i5_24_003_entailment() {
    check_wg_case(&case_by_suffix("I5.24-2D003")).expect("should entail");
}

#[test]
fn wg_i5_5_005_entailment() {
    check_wg_case(&case_by_suffix("I5.5-2D005")).expect("should entail");
}

#[test]
fn wg_i5_8_006_entailment() {
    check_wg_case(&case_by_suffix("I5.8-2D006")).expect("should entail");
}

#[test]
fn wg_i5_8_017_entailment() {
    check_wg_case(&case_by_suffix("I5.8-2D017")).expect("should entail");
}

#[test]
fn wg_miscellaneous_001_consistent() {
    check_wg_case(&case_by_suffix("miscellaneous-2D001")).expect("consistent");
}

#[test]
fn wg_miscellaneous_002_consistent() {
    check_wg_case(&case_by_suffix("miscellaneous-2D002")).expect("consistent");
}
