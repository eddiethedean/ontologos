//! List all planned catalog axiom cases that fail semantic checks (engine gaps).
use ontologos_conformance::scan_planned_engine_failures;

fn main() {
    let failures = scan_planned_engine_failures();
    println!("planned engine failures: {}", failures.len());
    for (id, err) in &failures {
        println!("{id}");
        println!("  {err}");
    }
}
