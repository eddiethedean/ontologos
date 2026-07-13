//! SWRL HermiT RulesTest integration.

use ontologos_parser::load_ontology;
use std::path::Path;

fn hermit_ofn(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/axioms")
        .join(name)
}

#[test]
fn parses_swrl_rules_from_simple_rule() {
    let path = hermit_ofn("hermit_reasoner_rulestest_testsimplerule.ofn");
    let ontology = load_ontology(&path).expect("load");
    let rules = ontology.swrl_rules();
    assert_eq!(rules.len(), 1);
    let rule = &rules[0];
    assert_eq!(rule.body.len(), 1, "SimpleRule body atom count");
    assert_eq!(rule.head.len(), 1, "SimpleRule head atom count");
}

#[test]
fn same_as_in_body2_is_consistent_after_rules() {
    let path = hermit_ofn("hermit_reasoner_rulestest_testsameasinbody2.ofn");
    let mut ontology = load_ontology(&path).expect("load");
    assert!(!ontology.swrl_rules().is_empty());
    ontologos_swrl::apply_swrl_rules(&mut ontology).expect("apply");
    let consistent = ontologos_dl::is_consistent(&ontology).expect("consistent");
    assert!(consistent, "expected SAT after SWRL (distinct individuals)");
}

#[test]
fn same_as_in_body1_is_inconsistent_after_rules() {
    let path = hermit_ofn("hermit_reasoner_rulestest_testsameasinbody1.ofn");
    let mut ontology = load_ontology(&path).expect("load");
    let a = ontology.lookup_entity("file:/c/test.owl#a").expect("a");
    assert!(
        !ontology.object_assertions_of(a).is_empty(),
        "expected property assertions on a"
    );
    let report = ontologos_swrl::apply_swrl_rules(&mut ontology).expect("apply");
    assert!(
        report.inferences_added >= 1,
        "expected SWRL head to fire, got {:?}",
        report
    );
    let consistent = ontologos_dl::is_consistent(&ontology).expect("consistent");
    assert!(
        !consistent,
        "expected UNSAT: complement clash on same individual"
    );
}

#[test]
fn parses_datarange_swrl_rules_without_skips() {
    for name in [
        "hermit_reasoner_rulestest_testdatarangesafety.ofn",
        "hermit_reasoner_rulestest_testpositivebodydatarange.ofn",
        "hermit_reasoner_rulestest_testnegativebodydatarange.ofn",
        "hermit_reasoner_rulestest_testnegdrinhead.ofn",
        "hermit_reasoner_rulestest_testrulewithdatatypes2.ofn",
    ] {
        let path = hermit_ofn(name);
        let ontology = load_ontology(&path).unwrap_or_else(|e| panic!("{name}: {e}"));
        let meta = ontology.parse_meta().expect("meta");
        assert_eq!(
            meta.skipped_axiom_count, 0,
            "{name}: skipped {:?}",
            meta.warnings
        );
        assert!(
            !ontology.swrl_rules().is_empty(),
            "{name}: expected SWRL rules"
        );
    }
}

#[test]
fn simple_rule_materializes_class_assertion() {
    let path = hermit_ofn("hermit_reasoner_rulestest_testsimplerule.ofn");
    let mut ontology = load_ontology(&path).expect("load");
    ontologos_swrl::apply_swrl_rules(&mut ontology).expect("apply");
    let a = ontology.lookup_entity("file:/c/test.owl#a").expect("a");
    let c = ontology.lookup_entity("file:/c/test.owl#C").expect("C");
    assert!(
        ontology.classes_of(a).contains(&c),
        "expected a typed C from B(x)->C(x)"
    );
}
