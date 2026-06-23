//! Phase 3 ROADMAP priority consistency cases — gate for ALC tableau fixes.
use ontologos_dl::is_consistent;
use ontologos_parser::load_ontology;
use std::path::{Path, PathBuf};

fn ofn(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_reasonertest_{name}.ofn"
    ))
}

fn ofn_owl(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_owlreasonertest_{name}.ofn"
    ))
}

fn check(path: &Path, expected: bool, label: &str) {
    let ont = load_ontology(path).expect("load");
    let actual = is_consistent(&ont).expect("check");
    assert_eq!(
        actual, expected,
        "{label}: expected consistent={expected}, got {actual}"
    );
}

#[test]
fn testchains_is_inconsistent() {
    check(&ofn("testchains"), false, "testChains");
}

#[test]
fn testchains2_is_inconsistent() {
    check(&ofn("testchains2"), false, "testChains2");
}

#[test]
fn testinverses2_is_inconsistent() {
    check(&ofn("testinverses2"), false, "testInverses2");
}

#[test]
fn testinverses_is_inconsistent() {
    let path = ofn("testinverses");
    if !path.is_file() {
        return;
    }
    check(&path, false, "testInverses");
}

#[test]
fn testrole_disjointness_1_is_inconsistent() {
    check(
        &ofn("testroledisjointness_1"),
        false,
        "testRoleDisjointness_1",
    );
}

#[test]
fn testrole_disjointness_2_is_inconsistent() {
    check(
        &ofn("testroledisjointness_2"),
        false,
        "testRoleDisjointness_2",
    );
}

#[test]
fn testnegproperties_is_inconsistent() {
    check(&ofn("testnegproperties"), false, "testNegProperties");
}

#[test]
fn testnegative_data_property_assertion_is_inconsistent() {
    check(
        &ofn("testnegativedatapropertyassertion"),
        false,
        "testNegativeDataPropertyAssertion",
    );
}

#[test]
fn testincremental_addition2_is_inconsistent() {
    use ontologos_parser::load_ofn_with_incremental;
    let premise = ofn_owl("testincrementaladdition2");
    let incremental = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_owlreasonertest_testincrementaladdition2_incremental.ofn",
    );
    let ont = load_ofn_with_incremental(&premise, &incremental).expect("load merged");
    let actual = is_consistent(&ont).expect("check");
    assert!(
        !actual,
        "testIncrementalAddition2: expected inconsistent after incremental axioms"
    );
}
