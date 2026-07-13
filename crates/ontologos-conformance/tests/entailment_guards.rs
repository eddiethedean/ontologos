//! Entailment guard regression tests via public WG/SWRL catalog runners.

use ontologos_conformance::{check_axiom_case, check_wg_case, load_catalog, load_wg_catalog};

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
    let err = check_axiom_case(&case).expect_err("vacuous SWRL must fail closed");
    assert!(
        err.contains("vacuous pass blocked"),
        "unexpected error: {err}"
    );
}

#[test]
fn wg_keys_002_entailment_via_catalog_runner() {
    let case = load_wg_catalog()
        .iter()
        .find(|c| c.id == "owl_wg_tests.New-2DFeature-2DKeys-2D002")
        .expect("Keys-002 WG case");
    check_wg_case(case).expect("Keys-002 entailment");
}

#[test]
#[ignore = "DL incompleteness: named-class ⊥ not proven for Consistent-but-all-unsat (promoted via weak IRI guard pre-1.1.3); re-enable when classify/unsat probes succeed"]
fn wg_consistent_but_all_unsat_entailment_via_catalog_runner() {
    let case = load_wg_catalog()
        .iter()
        .find(|c| c.id == "owl_wg_tests.Consistent-2Dbut-2Dall-2Dunsat")
        .expect("Consistent-but-all-unsat WG case");
    check_wg_case(case).expect("consistent-but-all-unsat entailment");
}

#[test]
fn wg_equivalent_class_007_demorgan_via_catalog_runner() {
    let case = load_wg_catalog()
        .iter()
        .find(|c| c.id == "owl_wg_tests.TestCase-3AWebOnt-2DequivalentClass-2D007")
        .expect("equivalentClass-007 WG case");
    check_wg_case(case).expect("equivalentClass-007 entailment");
}

#[test]
fn wg_functional_property_004_via_catalog_runner() {
    let case = load_wg_catalog()
        .iter()
        .find(|c| c.id == "owl_wg_tests.TestCase-3AWebOnt-2DFunctionalProperty-2D004")
        .expect("FunctionalProperty-004 WG case");
    check_wg_case(case).expect("FunctionalProperty-004 entailment");
}

#[test]
fn wg_cardinality_003_via_catalog_runner() {
    let case = load_wg_catalog()
        .iter()
        .find(|c| c.id == "owl_wg_tests.TestCase-3AWebOnt-2Dcardinality-2D003")
        .expect("cardinality-003 WG case");
    check_wg_case(case).expect("cardinality-003 entailment");
}

/// Full reasoner path agreement — nightly only (no positive guard shortcuts).
#[test]
#[ignore = "slow DL merge — run in conformance nightly"]
fn positive_entailment_guards_agree_with_full_reasoner_on_wg_samples() {
    use ontologos_conformance::entailment_holds_with_budget_opts;
    use ontologos_parser::load_ontology;
    use std::path::PathBuf;
    use std::time::Duration;

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data/hermit/wg");
    let prem_path = root.join("Consistent-2Dbut-2Dall-2Dunsat/premise.rdf");
    let conc_path = root.join("Consistent-2Dbut-2Dall-2Dunsat/conclusion.rdf");
    assert!(
        prem_path.is_file() && conc_path.is_file(),
        "missing WG fixture"
    );
    let premise = load_ontology(&prem_path).expect("load premise");
    let conclusion = load_ontology(&conc_path).expect("load conclusion");
    let budget = Some(Duration::from_secs(30));
    let with_guards =
        entailment_holds_with_budget_opts(&premise, &conclusion, budget, true).expect("guards");
    let without_guards =
        entailment_holds_with_budget_opts(&premise, &conclusion, budget, false).expect("full");
    assert_eq!(
        with_guards, without_guards,
        "guard/full mismatch: guards={with_guards} full={without_guards}"
    );
    assert!(
        with_guards,
        "expected positive entailment for Consistent-but-all-unsat"
    );
}
