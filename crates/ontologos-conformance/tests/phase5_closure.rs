//! Phase 5 exit gate — Java planned backlog cleared, promotion candidates absorbed.

use ontologos_conformance::{
    audit_planned_backlog, load_catalog, parity_metrics, scan_promotable_axiom_cases,
    PlannedJavaCategory,
};

#[test]
fn phase5_java_planned_zero() {
    let metrics = parity_metrics();
    assert_eq!(
        metrics.java_planned, 0,
        "java_planned must be 0 for Phase 5 exit (got {})",
        metrics.java_planned
    );
}

#[test]
fn phase5_no_manual_port_in_audit() {
    let audit = audit_planned_backlog();
    let manual = audit
        .java
        .iter()
        .filter(|c| c.category == PlannedJavaCategory::ManualPort)
        .count();
    assert_eq!(manual, 0, "manual_port entries remain in planned backlog");
}

#[test]
fn phase5_promotion_candidates_empty_after_promote() {
    let pending = scan_promotable_axiom_cases();
    assert!(
        pending.is_empty(),
        "unpromoted passing planned cases:\n{}",
        pending.join("\n")
    );
}

#[test]
fn phase5_numerics_smoke() {
    let catalog = load_catalog();
    let case = catalog
        .iter()
        .find(|c| c.id == "reasoner.NumericsTest.testIntegerRange1")
        .expect("NumericsTest.testIntegerRange1 in catalog");
    ontologos_conformance::check_axiom_case(case).expect("numeric smoke check");
}
