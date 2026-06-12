use std::path::Path;

use ontologos_parser::load_ontology;
use ontologos_rdfs::RdfsEngine;
use ontologos_rl::RlEngine;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

#[test]
fn family_corpus_materializes_with_rl_inferences() {
    let path = repo_root().join("benchmarks/data/family.owl");
    assert!(
        path.exists(),
        "missing family corpus at {} (run ./benchmarks/scripts/download.sh)",
        path.display()
    );

    let mut rdfs_only = load_ontology(&path).expect("load family");
    let rdfs_report = RdfsEngine::new().materialize(&mut rdfs_only).expect("rdfs");
    let rdfs_total = rdfs_report.final_axiom_count;

    let mut ontology = load_ontology(&path).expect("load family");
    let report = RlEngine::new(1)
        .saturate(&mut ontology)
        .expect("rl saturate");

    assert!(report.final_axiom_count >= rdfs_total);
    assert!(
        report.inferred_total() > 0 || report.final_axiom_count > report.initial_axiom_count,
        "expected materialization via reasonable adapter"
    );
}
