use std::path::Path;

use ontologos_core::Axiom;
use ontologos_parser::load_ontology;
use ontologos_rl::RlEngine;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn family_corpus() -> std::path::PathBuf {
    let path = repo_root().join("benchmarks/data/family.owl");
    assert!(
        path.exists(),
        "missing family corpus at {} (run ./benchmarks/scripts/download.sh)",
        path.display()
    );
    path
}

fn has_property_range(ontology: &ontologos_core::Ontology, property: &str, range: &str) -> bool {
    let property = ontology.lookup_entity(property).expect("property");
    let range = ontology.lookup_entity(range).expect("range");
    ontology.axioms().iter().any(|(_, ax)| {
        matches!(
            ax,
            Axiom::ObjectPropertyRange {
                property: p,
                range: r
            } if *p == property && *r == range
        )
    })
}

#[test]
fn family_corpus_inherits_range_on_inverse_property() {
    let mut ontology = load_ontology(&family_corpus()).expect("load family");
    let report = RlEngine::new(1)
        .saturate(&mut ontology)
        .expect("rl saturate");

    assert!(
        report.inferred_total() > 0,
        "family RL should add inferences"
    );
    let ns = "http://a.com/ontology#";
    assert!(
        has_property_range(&ontology, &format!("{ns}hasChild"), &format!("{ns}Person")),
        "RL/RDFS should propagate hasParent range to hasChild via inverse"
    );
}

#[test]
fn family_corpus_adds_axioms_beyond_asserted_tbox() {
    let mut ontology = load_ontology(&family_corpus()).expect("load family");
    let initial = ontology.axiom_count();
    let report = RlEngine::new(1)
        .saturate(&mut ontology)
        .expect("rl saturate");

    assert!(report.final_axiom_count > initial);
    assert!(
        report.inferred_total() > 0,
        "family RL saturation should materialize new axioms"
    );
}
