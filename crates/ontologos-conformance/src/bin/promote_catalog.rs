//! Scan HermiT catalog axiom cases and write the full passing set for promotion.
use std::collections::BTreeSet;

use ontologos_conformance::{
    promoted_axiom_ids_path, read_catalog_file, read_promoted_axiom_ids,
    scan_all_passing_axiom_cases, scan_promotable_axiom_cases, scan_unpromoted_passing_axiom_cases,
    write_promoted_axiom_ids,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let incremental = args.iter().any(|a| a == "--incremental");

    let previous = read_promoted_axiom_ids();
    let passing = if incremental {
        let mut merged: BTreeSet<String> = previous.iter().cloned().collect();
        for id in scan_unpromoted_passing_axiom_cases() {
            merged.insert(id);
        }
        for id in scan_promotable_axiom_cases() {
            merged.insert(id);
        }
        merged.into_iter().collect()
    } else {
        scan_all_passing_axiom_cases()
    };

    let planned: BTreeSet<String> = read_catalog_file()
        .iter()
        .filter(|case| case.status == "planned")
        .map(|case| case.id.clone())
        .collect();
    let newly_planned: Vec<String> = passing
        .iter()
        .filter(|id| planned.contains(*id) && !previous.contains(*id))
        .cloned()
        .collect();
    let newly_unpromoted: usize = passing
        .iter()
        .filter(|id| !previous.contains(*id) && !planned.contains(*id))
        .count();

    println!(
        "newly promotable planned axiom cases: {}",
        newly_planned.len()
    );
    for id in &newly_planned {
        println!("  {id}");
    }
    if incremental {
        println!("newly passing unpromoted axiom cases: {newly_unpromoted}");
    }
    println!("all passing axiom cases: {}", passing.len());
    write_promoted_axiom_ids(&passing).expect("write promoted_axiom_ids.txt");
    println!("wrote {}", promoted_axiom_ids_path().display());
}
