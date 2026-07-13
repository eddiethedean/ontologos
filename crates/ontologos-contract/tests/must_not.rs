//! Rust integration contract MUST-NOT tests (see docs/guides/rust-integration-contract.md).

use ontologos_core::{Error, Ontology, Profile, Reasoner};
use ontologos_facade::{ClassifyOutcome, classify};
use std::path::PathBuf;

#[test]
#[allow(deprecated)]
fn ontology_from_file_returns_parse_not_available() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data/family.owl");
    assert!(path.is_file(), "missing family.owl fixture");
    let err = Ontology::from_file(&path).expect_err("from_file must not parse OWL");
    assert_eq!(err, Error::ParseNotAvailable);
}

#[test]
fn production_classify_routes_through_facade_not_core_stubs() {
    let ontology = Ontology::builder()
        .class("http://example.org/A")
        .unwrap()
        .class("http://example.org/B")
        .unwrap()
        .subclass_of("http://example.org/A", "http://example.org/B")
        .unwrap()
        .build()
        .unwrap();
    let mut reasoner = Reasoner::builder()
        .profile(Profile::El)
        .build(ontology)
        .unwrap();
    let outcome = classify(&mut reasoner).expect("facade classify");
    match outcome {
        ClassifyOutcome::Taxonomy(t) => {
            let a = reasoner
                .ontology()
                .lookup_entity("http://example.org/A")
                .unwrap();
            let b = reasoner
                .ontology()
                .lookup_entity("http://example.org/B")
                .unwrap();
            assert!(t.is_subsumed(a, b));
        }
        other => panic!("EL fixture must classify to taxonomy, got {other:?}"),
    }
}
