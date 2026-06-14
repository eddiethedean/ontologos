use ontologos_dl::is_consistent;
use ontologos_parser::load_ontology;
use std::path::PathBuf;

fn axiom(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/axioms")
        .join(name)
}

#[test]
fn exists_self1_is_inconsistent() {
    let ont = load_ontology(&axiom("hermit_reasoner_reasonertest_testexistsself1.ofn")).unwrap();
    assert!(!is_consistent(&ont).unwrap());
}

// exists_self2: nested HasSelf through a role cycle (self-loop + reuse existing r-successors).

#[test]
fn nominals1_is_consistent() {
    let ont = load_ontology(&axiom("hermit_reasoner_reasonertest_testnominals1.ofn")).unwrap();
    assert!(is_consistent(&ont).unwrap());
}

#[test]
fn nominals2_is_consistent() {
    let ont = load_ontology(&axiom("hermit_reasoner_reasonertest_testnominals2.ofn")).unwrap();
    assert!(is_consistent(&ont).unwrap());
}

#[test]
fn nominals3_is_consistent() {
    let ont = load_ontology(&axiom("hermit_reasoner_reasonertest_testnominals3.ofn")).unwrap();
    assert!(is_consistent(&ont).unwrap());
}

#[test]
fn nominals4_is_consistent() {
    let ont = load_ontology(&axiom("hermit_reasoner_reasonertest_testnominals4.ofn")).unwrap();
    assert!(is_consistent(&ont).unwrap());
}

#[test]
fn nominals5_is_consistent() {
    let ont = load_ontology(&axiom("hermit_reasoner_reasonertest_testnominals5.ofn")).unwrap();
    assert!(is_consistent(&ont).unwrap());
}

#[test]
fn nominals6_is_consistent() {
    let ont = load_ontology(&axiom("hermit_reasoner_reasonertest_testnominals6.ofn")).unwrap();
    assert!(is_consistent(&ont).unwrap());
}

#[test]
fn exists_self2_is_consistent() {
    let ont = load_ontology(&axiom("hermit_reasoner_reasonertest_testexistsself2.ofn")).unwrap();
    assert!(is_consistent(&ont).unwrap());
}

#[test]
fn nominal_merging_is_consistent() {
    let ont = load_ontology(&axiom("hermit_reasoner_reasonertest_testnominalmerging.ofn")).unwrap();
    assert!(is_consistent(&ont).unwrap());
}
