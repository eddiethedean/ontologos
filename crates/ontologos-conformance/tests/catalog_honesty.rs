//! Catalog inventory honesty — surfaced counts for burn-down tracking.

use ontologos_conformance::{load_catalog, load_wg_catalog};
use std::collections::HashSet;

#[test]
fn catalog_reports_non_generated_case_statuses() {
    let axiom_cases = load_catalog();
    let wg_cases = load_wg_catalog();

    let mut axiom_status: HashSet<&str> = HashSet::new();
    for case in &axiom_cases {
        axiom_status.insert(case.status.as_str());
    }
    let mut wg_status: HashSet<&str> = HashSet::new();
    for case in wg_cases {
        wg_status.insert(case.status.as_str());
    }

    let covered_axiom = axiom_cases.iter().filter(|c| c.status == "covered").count();
    let excluded_axiom = axiom_cases
        .iter()
        .filter(|c| c.status == "excluded")
        .count();
    let covered_wg = wg_cases.iter().filter(|c| c.status == "covered").count();
    let excluded_wg = wg_cases.iter().filter(|c| c.status == "excluded").count();

    eprintln!("axiom statuses: {axiom_status:?}");
    eprintln!("wg statuses: {wg_status:?}");
    eprintln!(
        "non-runnable inventory: axiom covered={covered_axiom} excluded={excluded_axiom}; wg covered={covered_wg} excluded={excluded_wg}"
    );

    assert!(
        covered_axiom + excluded_axiom + covered_wg + excluded_wg > 0,
        "catalog must track covered/excluded cases for burn-down"
    );
    assert!(
        axiom_cases.len() > 500,
        "axiom catalog should contain hundreds of HermiT cases"
    );
    assert!(
        wg_cases.len() > 400,
        "WG catalog should contain hundreds of cases"
    );
}
