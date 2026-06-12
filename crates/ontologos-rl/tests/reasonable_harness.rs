use std::path::Path;

use ontologos_parser::load_ontology;
use ontologos_rl::RlEngine;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

/// Smoke test for brick-subset fixture; full reasonable diff is manual via
/// `benchmarks/scripts/compare-reasonable.sh`.
#[test]
#[ignore = "optional external reasonable comparison; run compare-reasonable.sh manually"]
fn brick_subset_saturates() {
    let path = repo_root().join("benchmarks/data/brick-subset.ttl");
    assert!(path.exists(), "missing {}", path.display());
    let mut ontology = load_ontology(&path).expect("load brick subset");
    let initial = ontology.axiom_count();
    let report = RlEngine::new(1).saturate(&mut ontology).expect("saturate");
    assert!(report.final_axiom_count >= initial);
}
