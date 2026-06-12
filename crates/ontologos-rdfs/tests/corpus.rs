use std::path::Path;

use ontologos_parser::load_ontology;
use ontologos_rdfs::{RdfsEngine, RdfsRule};

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
    assert!(
        report
            .inferred_by_rule
            .get(&RdfsRule::RngInherit)
            .copied()
            .unwrap_or(0)
            > 0
            || report
                .inferred_by_rule
                .get(&RdfsRule::SpTrans)
                .copied()
                .unwrap_or(0)
                > 0
            || report
                .inferred_by_rule
                .get(&RdfsRule::DomInherit)
                .copied()
                .unwrap_or(0)
                > 0
            || report
                .inferred_by_rule
                .get(&RdfsRule::ScTrans)
                .copied()
                .unwrap_or(0)
                > 0,
        "expected at least one rule to fire: {:?}",
        report.inferred_by_rule
    );

    let base = "http://a.com/ontology#";
    let has_son = ontology
        .lookup_entity(&format!("{base}hasSon"))
        .expect("hasSon");
    let person = ontology
        .lookup_entity(&format!("{base}Person"))
        .expect("Person");
    assert!(
        ontology.index().ranges_of(has_son).contains(&person),
        "hasSon should inherit range Person from hasChild"
    );
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
