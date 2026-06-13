//! Targeted DL subsumption regression tests for ReasonerTest parity.

use ontologos_conformance::{check_axiom_case, load_catalog};

fn case_by_id(id: &str) -> ontologos_conformance::HermitCase {
    load_catalog()
        .iter()
        .find(|c| c.id == id)
        .cloned()
        .unwrap_or_else(|| panic!("missing case {id}"))
}

#[test]
fn reasoner_test_subsumption2_dl_or_rl() {
    let case = case_by_id("reasoner.ReasonerTest.testSubsumption2");
    check_axiom_case(&case).expect("testSubsumption2");
}

#[test]
fn reasoner_test_subsumption3_dl_or_rl() {
    let case = case_by_id("reasoner.ReasonerTest.testSubsumption3");
    check_axiom_case(&case).expect("testSubsumption3");
}

#[test]
fn reasoner_test_satisfiability_with_rias14() {
    let case = case_by_id("reasoner.ReasonerTest.testSatisfiabilityWithRIAs14");
    check_axiom_case(&case).expect("testSatisfiabilityWithRIAs14");
}

#[test]
fn reasoner_test_heinsohn_tbox3() {
    let case = case_by_id("reasoner.ReasonerTest.testHeinsohnTBox3");
    check_axiom_case(&case).expect("testHeinsohnTBox3");
}

#[test]
#[ignore = "complex cardinality flower ontology — tableau completion in progress"]
fn reasoner_test_classification_subclass_bug() {
    let case = case_by_id("reasoner.ReasonerTest.testClassificationSubClassBug");
    // Large flower ontology — allow slow tableau classification.
    check_axiom_case(&case).expect("testClassificationSubClassBug");
}
