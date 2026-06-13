//! List planned DL catalog cases that fail semantic checks (for triage).
use ontologos_conformance::{check_axiom_case, load_catalog};

fn main() {
    let mut failures: Vec<(String, String)> = Vec::new();
    for case in load_catalog() {
        if case.engine != "dl" || case.status != "planned" {
            continue;
        }
        if case.axiom_ofn.is_none() {
            continue;
        }
        if case.subsumptions.is_empty() && case.consistent.is_none() {
            continue;
        }
        if let Err(e) = check_axiom_case(case) {
            failures.push((case.id.clone(), e));
        }
    }
    failures.sort_by(|a, b| a.0.cmp(&b.0));
    println!("planned DL failures: {}", failures.len());
    for (id, err) in &failures {
        println!("{id}");
        println!("  {err}");
    }
}
