use std::path::Path;

use ontologos_parser::load_ontology;
use ontologos_rdfs::RdfsEngine;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

#[test]
fn family_corpus_materializes_with_rdfs_inferences() {
    let path = repo_root().join("benchmarks/data/family.owl");
    assert!(
        path.exists(),
        "missing family corpus at {} (run ./benchmarks/scripts/download.sh)",
        path.display()
    );

    let mut ontology = load_ontology(&path).expect("load family");
    let initial = ontology.axiom_count();
    let report = RdfsEngine::new()
        .materialize(&mut ontology)
        .expect("materialize family");

    assert!(report.final_axiom_count >= initial);
    assert!(report.inferred_total() > 0, "expected new RDFS inferences");
}

#[test]
fn pizza_corpus_materialized_is_superset_of_parsed_axioms() {
    let path = repo_root().join("benchmarks/data/pizza.owl");
    assert!(
        path.exists(),
        "missing pizza corpus at {} (run ./benchmarks/scripts/download.sh)",
        path.display()
    );

    let mut ontology = load_ontology(&path).expect("load pizza");
    let initial = ontology.axiom_count();
    let report = RdfsEngine::new()
        .materialize(&mut ontology)
        .expect("materialize pizza");

    assert!(
        report.final_axiom_count >= initial,
        "materialized pizza must be a superset of parsed axioms"
    );
}
