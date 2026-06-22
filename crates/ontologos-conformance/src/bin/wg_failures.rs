//! List planned OWL WG catalog cases that fail semantic checks (for triage).
use ontologos_conformance::scan_planned_wg_failures;

fn main() {
    let failures = scan_planned_wg_failures();
    println!("planned WG failures: {}", failures.len());
    for (id, err) in &failures {
        println!("{id}");
        println!("  {err}");
    }
}
