//! Scan planned HermiT catalog cases and promote those that pass semantic checks.
use ontologos_conformance::{
    promoted_axiom_ids_path, scan_promotable_axiom_cases, write_promoted_axiom_ids,
};

fn main() {
    let passing = scan_promotable_axiom_cases();
    println!("promotable planned axiom cases: {}", passing.len());
    for id in &passing {
        println!("  {id}");
    }
    write_promoted_axiom_ids(&passing).expect("write promoted_axiom_ids.txt");
    println!("wrote {}", promoted_axiom_ids_path().display());
}
