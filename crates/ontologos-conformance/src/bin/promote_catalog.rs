//! Scan HermiT catalog axiom cases and write the full passing set for promotion.
use std::collections::HashSet;

use ontologos_conformance::{
    promoted_axiom_ids_path, read_catalog_file, read_promoted_axiom_ids,
    scan_all_passing_axiom_cases, write_promoted_axiom_ids,
};

fn main() {
    let previous = read_promoted_axiom_ids();
    let passing = scan_all_passing_axiom_cases();
    let planned: HashSet<String> = read_catalog_file()
        .iter()
        .filter(|case| case.status == "planned")
        .map(|case| case.id.clone())
        .collect();
    let newly_planned: Vec<String> = passing
        .iter()
        .filter(|id| planned.contains(*id) && !previous.contains(*id))
        .cloned()
        .collect();

    println!(
        "newly promotable planned axiom cases: {}",
        newly_planned.len()
    );
    for id in &newly_planned {
        println!("  {id}");
    }
    println!("all passing axiom cases: {}", passing.len());
    write_promoted_axiom_ids(&passing).expect("write promoted_axiom_ids.txt");
    println!("wrote {}", promoted_axiom_ids_path().display());
}
