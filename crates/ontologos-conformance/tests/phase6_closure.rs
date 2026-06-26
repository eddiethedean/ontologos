//! Phase 6 exit gate — HermiT Tier B classification corpora in default CI.

use ontologos_conformance::{
    classification_fixture_path, classification_fixtures_available, load_catalog,
};

const TIER_B_IDS: &[&str] = &[
    "reasoner.ClassificationTest.testPizza",
    "reasoner.ClassificationTest.testWine",
    "reasoner.ClassificationTest.testGalenIansFullUndoctored",
    "reasoner.ClassificationTest.testPropreo",
];

const FIXTURE_PATHS: &[&str] = &[
    "reasoner/res/pizza.xml",
    "reasoner/res/pizza.xml.txt",
    "reasoner/res/wine.xml",
    "reasoner/res/wine.xml.txt",
    "reasoner/res/galen-ians-full-undoctored.xml",
    "reasoner/res/galen-ians-full-undoctored.xml.txt",
    "reasoner/res/propreo.xml",
    "reasoner/res/propreo.xml.txt",
];

#[test]
fn phase6_classification_fixtures_vendored() {
    for relative in FIXTURE_PATHS {
        assert!(
            classification_fixture_path(relative).is_some(),
            "missing vendored ClassificationTest fixture: {relative}"
        );
    }
    assert!(
        classification_fixtures_available(),
        "classification_fixtures_available() should be true when all Tier B XML fixtures exist"
    );
}

#[test]
fn phase6_catalog_tier_b_active() {
    let catalog = load_catalog();
    for id in TIER_B_IDS {
        let case = catalog
            .iter()
            .find(|c| c.id == *id)
            .unwrap_or_else(|| panic!("missing catalog entry {id}"));
        assert_eq!(case.tier, "B", "{id}: expected tier B");
        assert!(
            !matches!(case.status.as_str(), "planned" | "excluded"),
            "{id}: status {} blocks Tier B CI",
            case.status
        );
        assert!(
            case.hand_written,
            "{id}: expected hand_written port in hermit_el.rs"
        );
        assert!(
            case.rust_test.is_some(),
            "{id}: expected rust_test name for hand-written port"
        );
    }
}

#[test]
fn phase6_propreo_classification_smoke() {
    use ontologos_el::ElClassifier;
    use ontologos_parser::load_ontology;

    let path = classification_fixture_path("reasoner/res/propreo.xml").expect("propreo.xml");
    let ontology = load_ontology(&path).expect("load propreo");
    let taxonomy = ElClassifier::new()
        .classify(&ontology)
        .expect("classify propreo");
    assert!(
        taxonomy.subsumption_count() > 0,
        "propreo smoke: expected inferred subsumptions"
    );
}
