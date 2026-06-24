//! Scan vendored OWL WG cases and write the passing set for promotion.
use std::collections::BTreeSet;

use ontologos_conformance::{
    promoted_wg_ids_path, read_wg_catalog_file, scan_all_passing_wg_cases,
    scan_planned_passing_wg_cases, write_promoted_wg_ids,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let incremental = args.iter().any(|a| a == "--incremental");

    let passing = if incremental {
        let existing: BTreeSet<String> = read_wg_catalog_file()
            .iter()
            .filter(|c| c.status == "wg")
            .map(|c| c.id.clone())
            .collect();
        let mut merged: BTreeSet<String> = existing;
        for id in scan_planned_passing_wg_cases() {
            merged.insert(id);
        }
        merged.into_iter().collect()
    } else {
        scan_all_passing_wg_cases()
    };

    println!("passing WG cases: {}", passing.len());
    for id in &passing {
        println!("  {id}");
    }
    write_promoted_wg_ids(&passing).expect("write promoted_wg_ids.txt");
    println!("wrote {}", promoted_wg_ids_path().display());
}
