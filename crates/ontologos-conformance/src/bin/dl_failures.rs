//! List planned DL catalog cases that fail semantic checks (for triage).
use ontologos_conformance::scan_planned_dl_failures;

fn main() {
    let failures = scan_planned_dl_failures();
    println!("planned DL failures: {}", failures.len());
    for (id, err) in &failures {
        println!("{id}");
        println!("  {err}");
    }
}
