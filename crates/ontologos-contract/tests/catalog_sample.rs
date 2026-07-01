//! Stratified catalog sample routed through the user contract runner.

use ontologos_conformance::{
    check_user_axiom_case, check_user_wg_case, load_catalog, load_wg_catalog, user_case_supported,
    user_wg_case_supported,
};
use std::path::PathBuf;

fn contract_case_ids() -> Vec<String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/case_ids.txt");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("missing contract case list {}: {e}", path.display()));
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

#[test]
fn contract_catalog_sample_axiom_cases() {
    let catalog = load_catalog();
    let by_id: std::collections::HashMap<_, _> =
        catalog.iter().map(|c| (c.id.as_str(), c)).collect();
    let mut ran = 0usize;
    for entry in contract_case_ids() {
        if entry.strip_prefix("wg:").is_some() {
            continue;
        }
        let case = by_id
            .get(entry.as_str())
            .unwrap_or_else(|| panic!("unknown contract case id: {entry}"));
        assert!(
            user_case_supported(case),
            "contract list includes unsupported case {entry}"
        );
        check_user_axiom_case(case).unwrap_or_else(|e| panic!("{entry}: {e}"));
        ran += 1;
    }
    assert!(
        ran >= 50,
        "expected at least 50 axiom contract cases, ran {ran}"
    );
}

#[test]
fn contract_catalog_sample_wg_cases() {
    let wg_catalog = load_wg_catalog();
    let by_id: std::collections::HashMap<_, _> =
        wg_catalog.iter().map(|c| (c.id.as_str(), c)).collect();
    let mut ran = 0usize;
    for entry in contract_case_ids() {
        let Some(wg_id) = entry.strip_prefix("wg:") else {
            continue;
        };
        let case = by_id
            .get(wg_id)
            .unwrap_or_else(|| panic!("unknown WG contract case id: {wg_id}"));
        assert!(
            user_wg_case_supported(case),
            "contract list includes unsupported WG case {wg_id}"
        );
        check_user_wg_case(case).unwrap_or_else(|e| panic!("{wg_id}: {e}"));
        ran += 1;
    }
    assert!(ran >= 5, "expected at least 5 WG contract cases, ran {ran}");
}
