//! Regression: catalog scans must not hang on pathological DL inputs.
use std::time::{Duration, Instant};

use ontologos_conformance::{check_axiom_case, load_catalog, scan_planned_engine_failures};

#[test]
fn planned_engine_failure_scan_completes_within_budget() {
    let start = Instant::now();
    let failures = scan_planned_engine_failures();
    let elapsed = start.elapsed();
    println!(
        "scan_planned_engine_failures: {} failures in {elapsed:?}",
        failures.len()
    );
    assert!(
        elapsed < Duration::from_secs(120),
        "planned scan took {elapsed:?} — likely pathological classify hang"
    );
}

#[test]
fn ian_backjumping3_axiom_check_completes_within_budget() {
    let case = load_catalog()
        .iter()
        .find(|c| c.id == "reasoner.ReasonerTest.testIanBackjumping3")
        .expect("catalog case");
    let start = Instant::now();
    let _ = check_axiom_case(case);
    let elapsed = start.elapsed();
    println!("testIanBackjumping3 check_axiom_case in {elapsed:?}");
    assert!(
        elapsed < Duration::from_secs(35),
        "testIanBackjumping3 took {elapsed:?} — classify budget should cap this"
    );
}
