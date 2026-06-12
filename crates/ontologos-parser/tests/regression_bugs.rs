//! Regression tests for confirmed bugs tracked on GitHub.
//! Run ignored tests: `cargo test -p ontologos-parser -- --ignored`

use std::path::Path;

use ontologos_core::{EntityKind, ParseMeta};
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

const NS: &str = "http://example.org/test";

fn entity_iri(local: &str) -> String {
    format!("{NS}#{local}")
}

fn entity_kind(ontology: &ontologos_core::Ontology, local: &str) -> EntityKind {
    let id = ontology
        .lookup_entity(&entity_iri(local))
        .unwrap_or_else(|| panic!("missing entity {local}"));
    ontology.entities().entity(id).expect("entity record").kind
}

fn assert_subclass_data_property_conflict(ontology: &ontologos_core::Ontology) -> ParseMeta {
    assert_eq!(ontology.axiom_count(), 0, "SubClassOf should be skipped");
    assert_eq!(entity_kind(ontology, "X"), EntityKind::DataProperty);
    assert_eq!(entity_kind(ontology, "Y"), EntityKind::Class);
    let meta = ontology.parse_meta().expect("parse_meta").clone();
    assert_eq!(meta.mapped_axiom_count, 0);
    assert_eq!(meta.skipped_axiom_count, 1);
    assert_eq!(meta.logical_axiom_count, 1);
    assert_kind_mismatch_without_misleading_skip(&meta, &["complex class expression"]);
    meta
}

/// SubClassOf(:Y :X) with :X declared DataProperty must not depend on axiom visit order.
#[test]
fn subclass_data_property_conflict_is_order_independent_decl_first() {
    let ontology = load_ontology(&fixture("subclass_data_property_decl_first.ofn")).expect("load");
    assert_subclass_data_property_conflict(&ontology);
}

#[test]
fn subclass_data_property_conflict_is_order_independent_axiom_first() {
    let ontology = load_ontology(&fixture("subclass_data_property_axiom_first.ofn")).expect("load");
    assert_subclass_data_property_conflict(&ontology);
}

#[test]
fn subclass_data_property_conflict_ofn_orderings_share_parse_meta() {
    let decl_first =
        load_ontology(&fixture("subclass_data_property_decl_first.ofn")).expect("load");
    let axiom_first =
        load_ontology(&fixture("subclass_data_property_axiom_first.ofn")).expect("load");
    let meta_decl = assert_subclass_data_property_conflict(&decl_first);
    let meta_axiom = assert_subclass_data_property_conflict(&axiom_first);
    assert_eq!(meta_decl.warnings.len(), meta_axiom.warnings.len());
    assert_eq!(meta_decl.warnings, meta_axiom.warnings);
}

#[test]
fn subclass_data_property_conflict_is_order_independent_decl_first_turtle() {
    let ontology = load_ontology(&fixture("subclass_data_property_decl_first.ttl")).expect("load");
    assert_subclass_data_property_conflict(&ontology);
}

#[test]
fn subclass_data_property_conflict_is_order_independent_axiom_first_turtle() {
    let ontology = load_ontology(&fixture("subclass_data_property_axiom_first.ttl")).expect("load");
    assert_subclass_data_property_conflict(&ontology);
}

#[test]
fn subclass_data_property_conflict_turtle_orderings_share_parse_meta() {
    let decl_first =
        load_ontology(&fixture("subclass_data_property_decl_first.ttl")).expect("load");
    let axiom_first =
        load_ontology(&fixture("subclass_data_property_axiom_first.ttl")).expect("load");
    let meta_decl = assert_subclass_data_property_conflict(&decl_first);
    let meta_axiom = assert_subclass_data_property_conflict(&axiom_first);
    assert_eq!(meta_decl.warnings.len(), meta_axiom.warnings.len());
    assert_eq!(meta_decl.warnings, meta_axiom.warnings);
}

#[test]
fn subclass_named_classes_still_maps_when_declarations_precede_axiom() {
    let ontology = load_ontology(&fixture("subclass_named_classes.ofn")).expect("load");
    assert_eq!(ontology.axiom_count(), 1);
    assert_eq!(entity_kind(&ontology, "X"), EntityKind::Class);
    assert_eq!(entity_kind(&ontology, "Y"), EntityKind::Class);
}
