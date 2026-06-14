//! Report DL OFN fixture semantic pass rate from the HermiT catalog.

use std::collections::BTreeMap;
use std::path::PathBuf;

use ontologos_conformance::{load_catalog, HermitCase};
use ontologos_core::Ontology;
use ontologos_parser::load_ontology;
use rayon::prelude::*;

fn hermit_data_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit")
        .join(rel)
}

fn resolve_local_iri(local: &str) -> String {
    const NS: &str = "file:/c/test.owl#";
    if local.contains("://") || local.starts_with("file:") {
        local.to_owned()
    } else {
        let name = local.strip_prefix(':').unwrap_or(local);
        format!("{NS}{name}")
    }
}

fn case_passes(case: &HermitCase, ontology: &Ontology) -> bool {
    if case.engine != "dl" && case.engine != "alc" {
        return false;
    }

    if let Some(expected) = case.consistent {
        let Ok(actual) = ontologos_dl::is_consistent(ontology) else {
            return false;
        };
        if actual != expected {
            return false;
        }
    }

    if !case.subsumptions.is_empty() {
        let Ok(taxonomy) = ontologos_dl::classify(ontology) else {
            return false;
        };
        for sub in &case.subsumptions {
            let sub_iri = resolve_local_iri(&sub.sub);
            let sup_iri = resolve_local_iri(&sub.sup);
            let Some(sub_id) = ontology.lookup_entity(&sub_iri) else {
                return false;
            };
            let Some(sup_id) = ontology.lookup_entity(&sup_iri) else {
                return false;
            };
            if taxonomy.is_subsumed(sub_id, sup_id) != sub.expected {
                return false;
            }
        }
    }

    if case.subsumptions.is_empty() && case.consistent.is_none() {
        return false;
    }

    true
}

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

    let path = hermit_data_path(ofn_rel);
    if !path.is_file() {
        return Some(CaseOutcome {
            family: family(&case.java_class),
            passed: false,
            skipped: true,
        });
    }

    let Ok(ontology) = load_ontology(&path) else {
        return Some(CaseOutcome {
            family: family(&case.java_class),
            passed: false,
            skipped: false,
        });
    };

    Some(CaseOutcome {
        family: family(&case.java_class),
        passed: case_passes(case, &ontology),
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
