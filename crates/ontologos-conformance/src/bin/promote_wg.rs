//! Scan vendored OWL WG cases and write the passing set for promotion.
use ontologos_conformance::{
    promoted_wg_ids_path, scan_all_passing_wg_cases, write_promoted_wg_ids,
};

fn main() {
    let passing = scan_all_passing_wg_cases();
    println!("passing WG cases: {}", passing.len());
    for id in &passing {
        println!("  {id}");
    }
    write_promoted_wg_ids(&passing).expect("write promoted_wg_ids.txt");
    println!("wrote {}", promoted_wg_ids_path().display());
}
