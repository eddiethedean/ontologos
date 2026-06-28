//! Phase 3 exit gate — engine_gap cleared, promotion candidates absorbed.

use ontologos_conformance::{
    audit_planned_backlog, load_catalog, scan_planned_engine_failures, scan_promotable_axiom_cases,
    PlannedJavaCategory,
};

#[test]
fn phase3_engine_failures_empty() {
    let failures = scan_planned_engine_failures();
    assert!(
        failures.is_empty(),
        "planned engine failures: {}",
        failures
            .iter()
            .map(|(id, err)| format!("{id}: {err}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn phase3_no_engine_gap_in_audit() {
    let audit = audit_planned_backlog();
    let engine_gap = audit
        .java
        .iter()
        .filter(|c| c.category == PlannedJavaCategory::EngineGap)
        .count();
    assert_eq!(engine_gap, 0, "engine_gap cases remain in planned backlog");
}

#[test]
fn phase3_promoted_reasoner_smoke_passes() {
    let catalog = load_catalog();
    let case = catalog
        .iter()
        .find(|c| c.id == "reasoner.AnyURITest.testIntersection")
        .expect("AnyURITest.testIntersection in catalog");
    ontologos_conformance::check_axiom_case(case).expect("promoted reasoner smoke check");
}

#[test]
fn phase3_promotion_candidates_empty_after_promote() {
    let pending = scan_promotable_axiom_cases();
    assert!(
        pending.is_empty(),
        "unpromoted passing planned cases:\n{}",
        pending.join("\n")
    );
}
