//! Entailment guard regressions via the user contract runner.

use ontologos_conformance::{
    check_user_axiom_case, check_user_wg_case, load_catalog, load_wg_catalog,
};

#[test]
fn swrl_case_without_assertions_is_rejected() {
    let mut case = load_catalog()
        .into_iter()
        .find(|c| c.id == "reasoner.RulesTest.testSimpleRule")
        .expect("RulesTest.testSimpleRule in catalog");
    case.individual_types.clear();
    case.consistent = None;
    case.subsumptions.clear();
    case.class_satisfiability.clear();
    case.conclusion_ofn = None;
    case.expected_entailment = None;
    let err = check_user_axiom_case(&case).expect_err("vacuous SWRL must fail closed");
    assert!(
        err.contains("vacuous pass blocked"),
        "unexpected error: {err}"
    );
}

#[test]
fn wg_keys_002_entailment_via_user_runner() {
    let case = load_wg_catalog()
        .iter()
        .find(|c| c.id == "owl_wg_tests.New-2DFeature-2DKeys-2D002")
        .expect("Keys-002 WG case");
    check_user_wg_case(case).expect("Keys-002 entailment");
}

#[test]
fn wg_consistent_but_all_unsat_remains_deferred_via_user_runner() {
    let case = load_wg_catalog()
        .iter()
        .find(|c| c.id == "owl_wg_tests.Consistent-2Dbut-2Dall-2Dunsat")
        .expect("Consistent-but-all-unsat WG case");
    assert_eq!(
        case.status, "deferred",
        "re-activate in deferred_wg_ids.txt only after named-class ⊥ is proven"
    );
    let err = check_user_wg_case(case)
        .expect_err("deferred case must not pass via weak IRI shortcut");
    assert!(
        err.contains("entailment expected true")
            || err.contains("not supported")
            || err.contains("deferred"),
        "unexpected error: {err}"
    );
}

#[test]
fn wg_equivalent_class_007_demorgan_via_user_runner() {
    let case = load_wg_catalog()
        .iter()
        .find(|c| c.id == "owl_wg_tests.TestCase-3AWebOnt-2DequivalentClass-2D007")
        .expect("equivalentClass-007 WG case");
    check_user_wg_case(case).expect("equivalentClass-007 entailment");
}
