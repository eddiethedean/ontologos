use ontologos_dl::is_consistent;
use ontologos_parser::load_ontology;
use std::path::PathBuf;

fn ofn(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_reasonertest_{name}.ofn"
    ))
}

fn check(name: &str, expected: bool) {
    let ont = load_ontology(&ofn(name)).expect("load");
    let actual = is_consistent(&ont).expect("check");
    assert_eq!(
        actual, expected,
        "{name}: expected {expected}, got {actual}"
    );
}

#[test]
fn chains_is_inconsistent() {
    check("testchains", false);
}

#[test]
fn inverses2_is_inconsistent() {
    check("testinverses2", false);
}

#[test]
fn reflexivity_is_inconsistent() {
    check("testreflexivity", false);
}

#[test]
fn bottom_data_property_is_inconsistent() {
    check("testbottomdataproperty", false);
}

#[test]
fn satisfiability2_is_inconsistent() {
    check("testsatisfiability2", false);
}

#[test]
fn satisfiability4_is_inconsistent() {
    check("testsatisfiability4", false);
}

#[test]
fn satisfiability3_is_inconsistent() {
    check("testsatisfiability3", false);
}

#[test]
fn universal_role_partitioned_abox_is_inconsistent() {
    check("testuniversalrolepartitionedabox", false);
}

#[test]
fn role_chains_with_transitive_symmetric_is_inconsistent() {
    check("testrolechainswithtransitivesymmetric", false);
}
