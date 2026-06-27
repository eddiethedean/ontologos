//! Resync promotion lists to exactly the passing axiom and WG case sets.
use ontologos_conformance::{
    ensure_concurrent_scan_defaults, read_promoted_axiom_ids, read_promoted_wg_ids,
    sync_promoted_lists,
};

fn main() {
    ensure_concurrent_scan_defaults();
    let previous_axiom = read_promoted_axiom_ids();
    let previous_wg = read_promoted_wg_ids();
    let (axiom, wg) = sync_promoted_lists();
    let removed_axiom = previous_axiom
        .iter()
        .filter(|id| !axiom.iter().any(|p| p == *id))
        .count();
    let removed_wg = previous_wg
        .iter()
        .filter(|id| !wg.iter().any(|p| p == *id))
        .count();
    println!("sync promoted lists (passing-only)");
    println!("  axiom: {} passing ({removed_axiom} demoted)", axiom.len());
    println!("  wg:    {} passing ({removed_wg} demoted)", wg.len());
}
