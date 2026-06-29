//! HermiT `tableau.*` engine-internal tests — inventory and deferred ports (Tier B3).
//!
//! HermiT tableau tests construct `DLOntology` + `Tableau` directly (merge/backtrack,
//! NI rules, blocking validators). Full ports require exposing HermiT-equivalent
//! extension-manager hooks; tracked here until B3 tableau burn-down.

/// Catalog `tableau.*` cases that remain `status = internal` in cases.json.
const TABLEAU_INTERNAL_IDS: &[&str] = &[
    "tableau.BlockingValidatorTest.testInvalidBlockWithAnnotatedEqualities",
    "tableau.BlockingValidatorTest.testOneInvalidBlock",
    "tableau.DLClauseEvaluationTest.testEvaluator",
    "tableau.DependencySetTest.testDependencySet1",
    "tableau.DependencySetTest.testDependencySet2",
    "tableau.DependencySetTest.testDependencySet3",
    "tableau.GraphTest.testGraph1",
    "tableau.GraphTest.testGraphMerging",
    "tableau.MergeTest.testMergeAndBacktrack",
    "tableau.NIRuleTest.testContentingNIs",
    "tableau.NIRuleTest.testDeterministicRuleApplication",
    "tableau.NIRuleTest.testDisjunctionDerivation",
    "tableau.NIRuleTest.testDisjunctionsInTreePart",
    "tableau.NIRuleTest.testNIAndPruning",
    "tableau.NIRuleTest.testNIDoesNotPrune",
    "tableau.NIRuleTest.testNIPrunesOneNode",
    "tableau.NIRuleTest.testNIRuleDeterministic",
    "tableau.NIRuleTest.testNondeterministicEquality",
    "tableau.NIRuleTest.testRepeatedNIApplications",
    "tableau.TupleIndexTest.testIndex1",
    "tableau.TupleIndexTest.testIndex2",
    "tableau.TupleTableFullIndexTest.testIndex",
    "tableau.TupleTableFullIndexTest.testLotsOfData",
];

#[test]
fn hermit_tableau_internal_inventory() {
    #[derive(serde::Deserialize)]
    struct Case {
        id: String,
        status: String,
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/catalog/cases.json");
    let cases: Vec<Case> =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("cases.json")).expect("parse");
    for id in TABLEAU_INTERNAL_IDS {
        let case = cases
            .iter()
            .find(|c| c.id == *id)
            .unwrap_or_else(|| panic!("missing catalog entry {id}"));
        assert_eq!(
            case.status, "internal",
            "{id} should remain internal until ported"
        );
    }
    assert_eq!(TABLEAU_INTERNAL_IDS.len(), 23);
}
