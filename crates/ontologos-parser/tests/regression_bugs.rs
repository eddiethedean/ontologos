//! Regression tests for confirmed parser bugs tracked on GitHub.
//! All tests run in CI by default.

use std::path::Path;

use ontologos_core::{EntityKind, ParseMeta};
use ontologos_parser::load_ontology_lenient;

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

/// Class/individual punning should map the assertion without kind-mismatch warnings.
#[test]
fn class_individual_punning_maps_class_assertion() {
    let ontology = load_ontology_lenient(&fixture("class_individual_kind_clash.ttl")).expect("load");
    let meta = ontology.parse_meta().expect("parse_meta");
    assert_eq!(meta.skipped_axiom_count, 0, "warnings: {:?}", meta.warnings);
    assert_eq!(entity_kind(&ontology, "alice"), EntityKind::ClassIndividual);
}

/// SubClassOf with a punned class/individual IRI should map without kind mismatch.
#[test]
fn subclass_individual_punning_maps_subclass_of() {
    let ontology = load_ontology_lenient(&fixture("subclass_individual_kind_clash.ttl")).expect("load");
    let meta = ontology.parse_meta().expect("parse_meta");
    assert_eq!(meta.skipped_axiom_count, 0, "warnings: {:?}", meta.warnings);
    assert_eq!(entity_kind(&ontology, "alice"), EntityKind::ClassIndividual);
}

/// ObjectPropertyAssertion with a punned class/property IRI should map without kind mismatch.
#[test]
fn property_class_punning_maps_object_property_assertion() {
    let ontology =
        load_ontology_lenient(&fixture("property_assertion_class_kind_clash.ttl")).expect("load");
    let meta = ontology.parse_meta().expect("parse_meta");
    assert_eq!(meta.skipped_axiom_count, 0, "warnings: {:?}", meta.warnings);
    assert_eq!(
        entity_kind(&ontology, "knows"),
        EntityKind::ClassObjectProperty
    );
}

/// InverseObjectProperties with a punned class/property IRI should map without kind mismatch.
#[test]
fn inverse_properties_class_punning_maps_inverse_axiom() {
    let ontology =
        load_ontology_lenient(&fixture("inverse_properties_class_kind_clash.ttl")).expect("load");
    let meta = ontology.parse_meta().expect("parse_meta");
    assert_eq!(meta.skipped_axiom_count, 0, "warnings: {:?}", meta.warnings);
    assert_eq!(
        entity_kind(&ontology, "parent"),
        EntityKind::ClassObjectProperty
    );
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
    let ontology = load_ontology_lenient(&fixture("subclass_data_property_decl_first.ofn")).expect("load");
    assert_subclass_data_property_conflict(&ontology);
}

#[test]
fn subclass_data_property_conflict_is_order_independent_axiom_first() {
    let ontology = load_ontology_lenient(&fixture("subclass_data_property_axiom_first.ofn")).expect("load");
    assert_subclass_data_property_conflict(&ontology);
}

#[test]
fn subclass_data_property_conflict_ofn_orderings_share_parse_meta() {
    let decl_first =
        load_ontology_lenient(&fixture("subclass_data_property_decl_first.ofn")).expect("load");
    let axiom_first =
        load_ontology_lenient(&fixture("subclass_data_property_axiom_first.ofn")).expect("load");
    let meta_decl = assert_subclass_data_property_conflict(&decl_first);
    let meta_axiom = assert_subclass_data_property_conflict(&axiom_first);
    assert_eq!(meta_decl.warnings.len(), meta_axiom.warnings.len());
    assert_eq!(meta_decl.warnings, meta_axiom.warnings);
}

#[test]
fn subclass_data_property_conflict_is_order_independent_decl_first_turtle() {
    let ontology = load_ontology_lenient(&fixture("subclass_data_property_decl_first.ttl")).expect("load");
    assert_subclass_data_property_conflict(&ontology);
}

#[test]
fn subclass_data_property_conflict_is_order_independent_axiom_first_turtle() {
    let ontology = load_ontology_lenient(&fixture("subclass_data_property_axiom_first.ttl")).expect("load");
    assert_subclass_data_property_conflict(&ontology);
}

#[test]
fn subclass_data_property_conflict_turtle_orderings_share_parse_meta() {
    let decl_first =
        load_ontology_lenient(&fixture("subclass_data_property_decl_first.ttl")).expect("load");
    let axiom_first =
        load_ontology_lenient(&fixture("subclass_data_property_axiom_first.ttl")).expect("load");
    let meta_decl = assert_subclass_data_property_conflict(&decl_first);
    let meta_axiom = assert_subclass_data_property_conflict(&axiom_first);
    assert_eq!(meta_decl.warnings.len(), meta_axiom.warnings.len());
    assert_eq!(meta_decl.warnings, meta_axiom.warnings);
}

#[test]
fn subclass_named_classes_still_maps_when_declarations_precede_axiom() {
    let ontology = load_ontology_lenient(&fixture("subclass_named_classes.ofn")).expect("load");
    assert_eq!(ontology.axiom_count(), 1);
    assert_eq!(entity_kind(&ontology, "X"), EntityKind::Class);
    assert_eq!(entity_kind(&ontology, "Y"), EntityKind::Class);
}

#[test]
fn invalid_blank_nodes_conclusion_rejected() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/axioms/hermit_reasoner_entailmenttest_testinvalidblanknodes_conclusion.ofn");
    let ontology = ontologos_parser::load_ontology_lenient(&path).expect("load");
    assert!(
        ontologos_parser::validate_loaded_ontology(&ontology).is_err(),
        "expected cyclic blank-node conclusion to fail validation"
    );
}
