use std::path::Path;
use std::time::Instant;

use ontologos_el::ElClassifier;
use ontologos_parser::load_ontology;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

#[test]
fn go_subset_classifies_within_budget() {
    let path = repo_root().join("benchmarks/data/go-subset.owl");
    if !path.exists() {
        eprintln!("skip: missing go-subset.owl (run benchmarks/scripts/generate-go-subset.sh)");
        return;
    }

    let ontology = load_ontology(&path).expect("load go-subset");
    let start = Instant::now();
    let taxonomy = ElClassifier::new()
        .classify(&ontology)
        .expect("classify go-subset");
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs() < 10,
        "go-subset classification took {:?}, budget is 10s",
        elapsed
    );
    assert!(taxonomy.subsumption_count() > 0);
}
