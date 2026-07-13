//! Corpus fixture smoke tests via the facade.

mod support;

use ontologos_core::{Profile, Reasoner};
use ontologos_facade::{ClassifyOutcome, classify};
use ontologos_parser::load_ontology;
use ontologos_profile::{OwlProfile, detect_profile};
use support::require_data_file;

#[test]
fn family_owl_auto_classify() {
    let path = require_data_file("family.owl");
    let ontology = load_ontology(&path).expect("load family.owl");
    let profile = detect_profile(&ontology).expect("detect profile");
    assert_eq!(
        profile.detected,
        Some(OwlProfile::Rl),
        "family.owl must detect as RL per profile-detection guide"
    );
    let mut reasoner = Reasoner::builder()
        .profile(Profile::Auto)
        .build(ontology)
        .expect("build");
    match classify(&mut reasoner).expect("classify") {
        ClassifyOutcome::Rl(r) => {
            assert!(
                r.inferred_total() > 0,
                "Family RL saturation must infer axioms"
            );
            let ns = "http://a.com/ontology#";
            let has_child = reasoner
                .ontology()
                .lookup_entity(&format!("{ns}hasChild"))
                .expect("hasChild");
            let person = reasoner
                .ontology()
                .lookup_entity(&format!("{ns}Person"))
                .expect("Person");
            let has_range = reasoner.ontology().axioms().iter().any(|(_, ax)| {
                matches!(
                    ax,
                    ontologos_core::Axiom::ObjectPropertyRange {
                        property: p,
                        range: r
                    } if *p == has_child && *r == person
                )
            });
            assert!(
                has_range,
                "RL must propagate hasParent range to hasChild via inverse (family corpus oracle)"
            );
        }
        other => panic!("Family Auto must route to RL materialization, got {other:?}"),
    }
}

#[test]
fn pizza_owl_el_classify() {
    let path = require_data_file("pizza.owl");
    let ontology = load_ontology(&path).expect("load pizza.owl");
    let mut reasoner = Reasoner::builder()
        .profile(Profile::El)
        .build(ontology)
        .expect("build");
    let outcome = classify(&mut reasoner).expect("classify");
    let tax = match outcome {
        ClassifyOutcome::Taxonomy(t) => t,
        other => panic!("EL pizza must return taxonomy, got {other:?}"),
    };
    let pizza_ns =
        "https://raw.githubusercontent.com/owlcs/pizza-ontology/refs/heads/master/pizza.owl#";
    let ham = reasoner
        .ontology()
        .lookup_entity(&format!("{pizza_ns}HamTopping"))
        .expect("HamTopping");
    let meat = reasoner
        .ontology()
        .lookup_entity(&format!("{pizza_ns}MeatTopping"))
        .expect("MeatTopping");
    assert!(
        tax.is_subsumed(ham, meat),
        "Pizza EL must infer HamTopping ⊑ MeatTopping"
    );
}
