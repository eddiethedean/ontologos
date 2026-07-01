use std::path::Path;

use ontologos_el::ElClassifier;
use ontologos_parser::load_ontology;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

#[test]
fn pizza_el_classification_produces_subsumptions() {
    let path = repo_root().join("benchmarks/data/pizza.owl");
    assert!(
        path.exists(),
        "missing {}; run ./benchmarks/scripts/download.sh",
        path.display()
    );

    let ontology = load_ontology(&path).expect("load pizza");
    let taxonomy = ElClassifier::new()
        .classify(&ontology)
        .expect("classify pizza EL");
    assert!(
        taxonomy.subsumption_count() >= 79,
        "expected ≥79 EL subsumptions from pizza corpus, got {}",
        taxonomy.subsumption_count()
    );

    let pizza_ns =
        "https://raw.githubusercontent.com/owlcs/pizza-ontology/refs/heads/master/pizza.owl#";
    let ham = ontology
        .lookup_entity(&format!("{pizza_ns}HamTopping"))
        .expect("HamTopping");
    let meat = ontology
        .lookup_entity(&format!("{pizza_ns}MeatTopping"))
        .expect("MeatTopping");
    assert!(taxonomy.is_subsumed(ham, meat), "HamTopping ⊑ MeatTopping");
}

#[test]
fn minimal_el_fixture_classifies() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../ontologos-parser/tests/fixtures/minimal_subclass.owl");
    let ontology = load_ontology(&path).expect("load");
    let taxonomy = ElClassifier::new().classify(&ontology).expect("classify");
    assert!(!taxonomy.subsumptions.is_empty());
}
