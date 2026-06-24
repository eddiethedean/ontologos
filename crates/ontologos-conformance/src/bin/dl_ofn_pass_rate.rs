//! Report DL OFN fixture semantic pass rate from the HermiT catalog.

use std::collections::BTreeMap;

use ontologos_conformance::{check_axiom_case_bounded, load_catalog, HermitCase};
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
    family: String,
    passed: bool,
    skipped: bool,
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
            family: family(&case.java_class),
            passed: false,
            skipped: true,
        });
    }

    Some(CaseOutcome {
        family: family(&case.java_class),
        passed: check_axiom_case_bounded(case).is_ok(),
        skipped: false,
    })
}

fn main() {
    let outcomes: Vec<CaseOutcome> = load_catalog()
        .par_iter()
        .filter_map(evaluate_case)
        .collect();

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
