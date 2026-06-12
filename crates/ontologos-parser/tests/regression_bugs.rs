//! Regression tests for confirmed bugs tracked on GitHub.
//! Run ignored tests: `cargo test -p ontologos-parser -- --ignored`

use std::path::Path;

use ontologos_core::ParseMeta;
use ontologos_parser::load_ontology;

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn assert_kind_mismatch_without_misleading_skip(meta: &ParseMeta, misleading: &[&str]) {
    assert!(
        meta.warnings
            .iter()
            .any(|w| w.contains("entity kind mismatch") || w.contains("EntityKindMismatch")),
        "expected kind mismatch warning, got: {:?}",
        meta.warnings
    );
    for fragment in misleading {
        assert!(
            !meta.warnings.iter().any(|w| w.contains(fragment)),
            "should not mislabel kind clash with {fragment:?}, got: {:?}",
            meta.warnings
        );
    }
}

/// Parser should report entity kind mismatch, not "complex operands" skip.
#[test]
fn class_assertion_kind_clash_surfaces_entity_kind_mismatch() {
    let ontology = load_ontology(&fixture("class_individual_kind_clash.ttl")).expect("load");
    let meta = ontology.parse_meta().expect("parse_meta");
    assert_kind_mismatch_without_misleading_skip(meta, &["complex operands"]);
}

/// SubClassOf with individual IRI used as subclass should surface kind mismatch.
#[test]
fn subclass_individual_kind_clash_surfaces_entity_kind_mismatch() {
    let ontology = load_ontology(&fixture("subclass_individual_kind_clash.ttl")).expect("load");
    let meta = ontology.parse_meta().expect("parse_meta");
    assert_kind_mismatch_without_misleading_skip(meta, &["complex class expression"]);
}

/// ObjectPropertyAssertion with class IRI used as property should surface kind mismatch.
#[test]
fn property_assertion_class_kind_clash_surfaces_entity_kind_mismatch() {
    let ontology =
        load_ontology(&fixture("property_assertion_class_kind_clash.ttl")).expect("load");
    let meta = ontology.parse_meta().expect("parse_meta");
    assert_kind_mismatch_without_misleading_skip(meta, &["complex operands"]);
}

/// InverseObjectProperties with class IRI used as property should surface kind mismatch.
#[test]
fn inverse_properties_class_kind_clash_surfaces_entity_kind_mismatch() {
    let ontology =
        load_ontology(&fixture("inverse_properties_class_kind_clash.ttl")).expect("load");
    let meta = ontology.parse_meta().expect("parse_meta");
    assert_kind_mismatch_without_misleading_skip(meta, &["unmapped operands"]);
}
