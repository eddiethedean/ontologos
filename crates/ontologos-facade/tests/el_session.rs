//! B-04: non-incremental EL classify clears stale incremental session.

use ontologos_core::{Ontology, Profile, Reasoner, ReasonerConfig};
use ontologos_facade::{classify, taxonomy_from_outcome};

fn chain_ontology() -> Ontology {
    Ontology::builder()
        .class("http://example.org/A")
        .unwrap()
        .class("http://example.org/B")
        .unwrap()
        .class("http://example.org/C")
        .unwrap()
        .subclass_of("http://example.org/A", "http://example.org/B")
        .unwrap()
        .subclass_of("http://example.org/B", "http://example.org/C")
        .unwrap()
        .build()
        .unwrap()
}

#[test]
fn non_incremental_el_after_incremental_uses_full_pass() {
    let ontology = chain_ontology();
    let mut incremental = Reasoner::builder()
        .profile(Profile::El)
        .config(ReasonerConfig {
            incremental: true,
            ..ReasonerConfig::default()
        })
        .build(ontology.clone())
        .unwrap();
    let _ = classify(&mut incremental).unwrap();

    let mut full = Reasoner::builder()
        .profile(Profile::El)
        .config(ReasonerConfig {
            incremental: false,
            ..ReasonerConfig::default()
        })
        .build(ontology)
        .unwrap();
    let outcome = classify(&mut full).unwrap();
    let tax = taxonomy_from_outcome(&outcome).unwrap();
    let a = full
        .ontology()
        .lookup_entity("http://example.org/A")
        .unwrap();
    let c = full
        .ontology()
        .lookup_entity("http://example.org/C")
        .unwrap();
    assert!(tax.is_subsumed(a, c));
    assert!(
        full.session().is_none(),
        "non-incremental EL clears session"
    );
}
