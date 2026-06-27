//! Phase 9 promotion hygiene — promoted lists must contain only passing cases.

use ontologos_conformance::{
    read_promoted_axiom_ids, read_promoted_wg_ids, scan_promoted_axiom_failures,
    scan_promoted_wg_failures,
};

#[test]
fn phase9_promoted_axiom_lists_match_passing() {
    let failures = scan_promoted_axiom_failures();
    assert!(
        failures.is_empty(),
        "promoted axiom failures ({} promoted ids):\n{}",
        read_promoted_axiom_ids().len(),
        failures
            .iter()
            .map(|(id, err)| format!("{id}: {err}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn phase9_promoted_wg_lists_match_passing() {
    let failures = scan_promoted_wg_failures();
    assert!(
        failures.is_empty(),
        "promoted WG failures ({} promoted ids):\n{}",
        read_promoted_wg_ids().len(),
        failures
            .iter()
            .map(|f| format!("{} [{:?}]: {}", f.id, f.bucket, f.detail))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
