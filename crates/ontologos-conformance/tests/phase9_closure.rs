//! Phase 9 promotion hygiene — promoted lists must contain only passing cases.

use ontologos_conformance::{
    parity_metrics, read_promoted_axiom_ids, read_promoted_wg_ids, scan_promoted_axiom_failures,
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

#[test]
fn phase9_true_parity_pct_is_100() {
    let metrics = parity_metrics();
    assert!(
        metrics.true_parity_pct >= 99.9,
        "true_parity_pct must reach 100% (got {:.1}%)",
        metrics.true_parity_pct
    );
    assert!(
        metrics.literal_green_pct >= 99.9,
        "literal_green_pct must cover full catalog (got {:.1}%)",
        metrics.literal_green_pct
    );
}
