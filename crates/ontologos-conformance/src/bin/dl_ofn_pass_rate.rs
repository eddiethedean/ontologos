//! Report DL OFN fixture semantic pass rate from the HermiT catalog.

use std::collections::BTreeMap;

use ontologos_conformance::{HermitCase, check_axiom_case_bounded, load_catalog};
use rayon::prelude::*;

fn family(java_class: &str) -> String {
    java_class
        .rsplit('.')
        .next()
        .unwrap_or(java_class)
        .to_owned()
}

#[derive(Debug)]
struct CaseOutcome {
    id: String,
    family: String,
    passed: bool,
    skipped: bool,
    error: Option<String>,
}

fn evaluate_case(case: &HermitCase) -> Option<CaseOutcome> {
    if case.engine != "dl" {
        return None;
    }
    let ofn_rel = case.axiom_ofn.as_ref()?;
    if case.subsumptions.is_empty() && case.consistent.is_none() {
        return None;
    }

    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit")
        .join(ofn_rel);
    let skipped = !path.is_file();

    if skipped {
        return Some(CaseOutcome {
            id: case.id.clone(),
            family: family(&case.java_class),
            passed: false,
            skipped: true,
            error: None,
        });
    }

    let check = check_axiom_case_bounded(case);
    Some(CaseOutcome {
        id: case.id.clone(),
        family: family(&case.java_class),
        passed: check.is_ok(),
        skipped: false,
        error: check.err().map(|e| e.to_string()),
    })
}

fn main() {
    let list_failures = std::env::args().any(|a| a == "--failures");
    let outcomes: Vec<CaseOutcome> = load_catalog()
        .par_iter()
        .filter_map(evaluate_case)
        .collect();

    if list_failures {
        let mut failures: Vec<_> = outcomes
            .iter()
            .filter(|o| !o.skipped && !o.passed)
            .collect();
        failures.sort_by(|a, b| a.id.cmp(&b.id));
        let count = failures.len();
        for f in &failures {
            println!(
                "{} [{}] {}",
                f.id,
                f.family,
                f.error.as_deref().unwrap_or("?")
            );
        }
        eprintln!("failures: {count}");
        return;
    }

    let mut candidates = 0_u32;
    let mut passed = 0_u32;
    let mut skipped = 0_u32;
    let mut by_family: BTreeMap<String, (u32, u32)> = BTreeMap::new();

    for outcome in outcomes {
        if outcome.skipped {
            skipped += 1;
            continue;
        }
        candidates += 1;
        let entry = by_family.entry(outcome.family).or_insert((0, 0));
        entry.0 += 1;
        if outcome.passed {
            passed += 1;
            entry.1 += 1;
        }
    }

    println!("DL OFN semantic pass rate (catalog cases with assertions)");
    println!("  candidates: {candidates}");
    println!("  passed:     {passed}");
    println!("  skipped:    {skipped} (missing .ofn on disk)");
    if candidates > 0 {
        let pct = (passed as f64) * 100.0 / (candidates as f64);
        println!("  pass rate:  {passed}/{candidates} ({pct:.1}%)");
    }
    println!();
    println!("By Java class family:");
    for (fam, (total, ok)) in &by_family {
        let pct = if *total > 0 {
            (*ok as f64) * 100.0 / (*total as f64)
        } else {
            0.0
        };
        println!("  {fam}: {ok}/{total} ({pct:.1}%)");
    }
}
