//! Corpus fixture smoke tests via the facade.

use ontologos_core::{Profile, Reasoner};
use ontologos_facade::{classify, taxonomy_from_outcome, ClassifyOutcome};
use ontologos_parser::load_ontology;
use std::path::PathBuf;

fn data_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data").join(name)
}

#[test]
fn family_owl_auto_classify() {
    let path = data_path("family.owl");
    if !path.is_file() {
        return;
    }
    let ontology = load_ontology(&path).expect("load family.owl");
    let mut reasoner = Reasoner::builder()
        .profile(Profile::Auto)
        .build(ontology)
        .expect("build");
    match classify(&mut reasoner).expect("classify") {
        ClassifyOutcome::Taxonomy(t) => {
            assert!(t.subsumption_count() > 0 || t.subsumptions.is_empty());
        }
        ClassifyOutcome::Rdfs(r) => {
            let _ = r.inferred_total();
        }
        ClassifyOutcome::Rl(r) => {
            assert!(r.inferred_total() > 0);
        }
    }
}

#[test]
fn pizza_owl_el_classify() {
    let path = data_path("pizza.owl");
    if !path.is_file() {
        return;
    }
    let ontology = load_ontology(&path).expect("load pizza.owl");
    let mut reasoner = Reasoner::builder()
        .profile(Profile::El)
        .build(ontology)
        .expect("build");
    let outcome = classify(&mut reasoner).expect("classify");
    let tax = taxonomy_from_outcome(&outcome).expect("EL pizza taxonomy");
    assert!(tax.subsumption_count() > 0);
}
