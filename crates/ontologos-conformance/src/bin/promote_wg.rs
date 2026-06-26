//! Scan vendored OWL WG cases and write the passing set for promotion.
use std::collections::BTreeSet;

use ontologos_conformance::{
    ensure_concurrent_scan_defaults, promoted_wg_ids_path, read_promoted_wg_ids,
    scan_all_passing_wg_cases, scan_unpromoted_passing_wg_cases, wg_case_short_id,
    write_promoted_wg_ids,
};

fn main() {
    ensure_concurrent_scan_defaults();
    let args: Vec<String> = std::env::args().collect();
    let incremental = args.iter().any(|a| a == "--incremental");

    let previous = read_promoted_wg_ids();
    let passing: Vec<String> = if incremental {
        let mut merged: BTreeSet<String> = previous.iter().cloned().collect();
        let newly_passing = scan_unpromoted_passing_wg_cases();
        println!("newly passing unpromoted WG: {}", newly_passing.len());
        for full_id in newly_passing {
            merged.insert(wg_case_short_id(&full_id).to_string());
        }
        merged.into_iter().collect()
    } else {
        scan_all_passing_wg_cases()
            .into_iter()
            .map(|id| wg_case_short_id(&id).to_string())
            .collect()
    };

    let added = passing.len().saturating_sub(previous.len());
    println!(
        "passing WG cases: {} (+{added} since last promote)",
        passing.len()
    );
    write_promoted_wg_ids(&passing).expect("write promoted_wg_ids.txt");
    println!("wrote {}", promoted_wg_ids_path().display());
}
