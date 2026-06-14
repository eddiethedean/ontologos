//! Scan HermiT catalog axiom cases and write the full passing set for promotion.
use ontologos_conformance::{
    promoted_axiom_ids_path, scan_all_passing_axiom_cases, scan_promotable_axiom_cases,
    write_promoted_axiom_ids,
};

fn main() {
    let newly_planned = scan_promotable_axiom_cases();
    let passing = scan_all_passing_axiom_cases();
    println!("newly promotable planned axiom cases: {}", newly_planned.len());
    for id in &newly_planned {
        println!("  {id}");
    }
    println!("all passing axiom cases: {}", passing.len());
    write_promoted_axiom_ids(&passing).expect("write promoted_axiom_ids.txt");
    println!("wrote {}", promoted_axiom_ids_path().display());
}
