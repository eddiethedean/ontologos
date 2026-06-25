//! Phase 4 exit gate — all active OWL WG cases pass at the DL budget.

use ontologos_conformance::{
    read_promoted_wg_ids, read_wg_catalog_file, scan_all_wg_failures, wg_case_short_id,
};

#[test]
fn phase4_wg_planned_zero() {
    let wg = read_wg_catalog_file();
    let planned = wg.iter().filter(|c| c.status == "planned").count();
    assert_eq!(planned, 0, "wg_planned must be 0 for Phase 4 exit");
}

#[test]
fn phase4_all_wg_failures_empty() {
    let failures = scan_all_wg_failures();
    assert!(
        failures.is_empty(),
        "active WG failures:\n{}",
        failures
            .iter()
            .map(|f| format!("{} [{:?}]: {}", f.id, f.bucket, f.detail))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn phase4_promoted_wg_complete() {
    let wg = read_wg_catalog_file();
    let active: Vec<_> = wg.iter().filter(|c| c.status == "wg").collect();
    let promoted = read_promoted_wg_ids();
    assert_eq!(
        promoted.len(),
        active.len(),
        "promoted_wg_ids.txt should list all {} active WG cases (got {})",
        active.len(),
        promoted.len()
    );
    for case in &active {
        let short = wg_case_short_id(&case.id);
        assert!(
            promoted.contains(short),
            "missing promoted id for {}",
            case.id
        );
    }
}
