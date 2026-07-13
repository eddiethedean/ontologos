//! Load shared semantic fixture expectations (benchmarks/data/semantic-fixtures.json).

use ontologos_core::{Profile, Reasoner};
use ontologos_facade::{ClassifyOutcome, classify};
use ontologos_parser::load_ontology;
use ontologos_profile::{OwlProfile, detect_profile};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
struct FixturesFile {
    el_minimal_subclass: ElMinimal,
    family_rl: FamilyRl,
    pizza_el: PizzaEl,
}

#[derive(Deserialize)]
struct ElMinimal {
    fixture: String,
    expected_subsumptions: Vec<[String; 2]>,
}

#[derive(Deserialize)]
struct FamilyRl {
    fixture: String,
    expected_profile_detect: String,
    expected_property_ranges_after_rl: Vec<[String; 2]>,
}

#[derive(Deserialize)]
struct PizzaEl {
    fixture: String,
    expected_subsumptions: Vec<[String; 2]>,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load_fixtures() -> FixturesFile {
    let path = repo_root().join("benchmarks/data/semantic-fixtures.json");
    let text = std::fs::read_to_string(&path).expect("semantic-fixtures.json");
    serde_json::from_str(&text).expect("parse semantic fixtures")
}

fn assert_pairs_in_taxonomy(
    ontology: &ontologos_core::Ontology,
    taxonomy: &ontologos_core::Taxonomy,
    pairs: &[[String; 2]],
) {
    for [sub, sup] in pairs {
        let sub_id = ontology.lookup_entity(sub).expect(sub);
        let sup_id = ontology.lookup_entity(sup).expect(sup);
        assert!(
            taxonomy.is_subsumed(sub_id, sup_id),
            "expected {sub} ⊑ {sup}"
        );
    }
}

#[test]
fn semantic_fixtures_el_minimal() {
    let fixtures = load_fixtures();
    let path = repo_root().join(&fixtures.el_minimal_subclass.fixture);
    assert!(path.is_file(), "missing {}", path.display());
    let ontology = load_ontology(&path).expect("load");
    let mut reasoner = Reasoner::builder()
        .profile(Profile::El)
        .build(ontology)
        .expect("build");
    let outcome = classify(&mut reasoner).expect("classify");
    match outcome {
        ClassifyOutcome::Taxonomy(t) => assert_pairs_in_taxonomy(
            reasoner.ontology(),
            &t,
            &fixtures.el_minimal_subclass.expected_subsumptions,
        ),
        other => panic!("expected taxonomy, got {other:?}"),
    }
}

#[test]
fn semantic_fixtures_family_rl() {
    let fixtures = load_fixtures();
    let path = repo_root().join(&fixtures.family_rl.fixture);
    assert!(path.is_file(), "missing {}", path.display());
    let ontology = load_ontology(&path).expect("load");
    let profile = detect_profile(&ontology).expect("detect");
    assert_eq!(
        profile.detected,
        Some(OwlProfile::Rl),
        "family profile oracle ({})",
        fixtures.family_rl.expected_profile_detect
    );
    let mut reasoner = Reasoner::builder()
        .profile(Profile::Rl)
        .build(ontology)
        .expect("build");
    let outcome = classify(&mut reasoner).expect("classify");
    assert!(
        matches!(outcome, ClassifyOutcome::Rl(_)),
        "family must RL-saturate"
    );
    for [property, range] in &fixtures.family_rl.expected_property_ranges_after_rl {
        let property_id = reasoner.ontology().lookup_entity(property).expect(property);
        let range_id = reasoner.ontology().lookup_entity(range).expect(range);
        let has_range = reasoner.ontology().axioms().iter().any(|(_, ax)| {
            matches!(
                ax,
                ontologos_core::Axiom::ObjectPropertyRange {
                    property: p,
                    range: r
                } if *p == property_id && *r == range_id
            )
        });
        assert!(has_range, "expected range {property} -> {range} after RL");
    }
}

#[test]
fn semantic_fixtures_pizza_el() {
    let fixtures = load_fixtures();
    let path = repo_root().join(&fixtures.pizza_el.fixture);
    assert!(
        path.is_file(),
        "missing {} — run download.sh",
        path.display()
    );
    let ontology = load_ontology(&path).expect("load");
    let mut reasoner = Reasoner::builder()
        .profile(Profile::El)
        .build(ontology)
        .expect("build");
    let outcome = classify(&mut reasoner).expect("classify");
    match outcome {
        ClassifyOutcome::Taxonomy(t) => assert_pairs_in_taxonomy(
            reasoner.ontology(),
            &t,
            &fixtures.pizza_el.expected_subsumptions,
        ),
        other => panic!("expected taxonomy, got {other:?}"),
    }
}
