use ontologos_core::{Ontology, OwlConstruct, ParseMeta};
use ontologos_profile::{OwlProfile, detect_profile};

fn ontology_with_profile_constructs(constructs: &[OwlConstruct]) -> Ontology {
    let mut meta = ParseMeta::default();
    for construct in constructs {
        meta.profile_constructs.insert(construct.clone());
        meta.constructs.insert(construct.clone());
    }
    let mut ontology = Ontology::new();
    ontology.set_parse_meta(meta);
    ontology
}

#[test]
fn empty_construct_set_detects_ql() {
    let ontology = Ontology::default();
    let report = detect_profile(&ontology).expect("detect");
    assert_eq!(report.detected, Some(OwlProfile::Ql));
}

#[test]
fn object_all_values_from_detects_ql() {
    let ontology = ontology_with_profile_constructs(&[OwlConstruct::ObjectAllValuesFrom]);
    let report = detect_profile(&ontology).expect("detect");
    assert_eq!(report.detected, Some(OwlProfile::Ql));
}

#[test]
fn el_only_mapped_set_detects_el() {
    let ontology = ontology_with_profile_constructs(&[
        OwlConstruct::SubClassOfExistential,
        OwlConstruct::ObjectSomeValuesFrom,
    ]);
    let report = detect_profile(&ontology).expect("detect");
    assert_eq!(report.detected, Some(OwlProfile::El));
}

#[test]
fn rl_marker_without_el_markers_detects_rl() {
    let ontology = ontology_with_profile_constructs(&[OwlConstruct::SymmetricObjectProperty]);
    let report = detect_profile(&ontology).expect("detect");
    assert_eq!(report.detected, Some(OwlProfile::Rl));
}

#[test]
fn both_el_and_rl_markers_prefers_el_when_el_markers_present() {
    let ontology = ontology_with_profile_constructs(&[
        OwlConstruct::SubClassOfExistential,
        OwlConstruct::ObjectSomeValuesFrom,
        OwlConstruct::TransitiveObjectProperty,
    ]);
    let report = detect_profile(&ontology).expect("detect");
    assert_eq!(report.detected, Some(OwlProfile::El));
}

#[test]
fn both_el_and_rl_markers_without_el_markers_prefers_rl() {
    let ontology = ontology_with_profile_constructs(&[
        OwlConstruct::TransitiveObjectProperty,
        OwlConstruct::SymmetricObjectProperty,
    ]);
    let report = detect_profile(&ontology).expect("detect");
    assert_eq!(report.detected, Some(OwlProfile::Rl));
}

#[test]
fn ql_forbidden_construct_escalates_to_dl() {
    let ontology = ontology_with_profile_constructs(&[OwlConstruct::ObjectUnionOf]);
    let report = detect_profile(&ontology).expect("detect");
    assert_eq!(report.detected, Some(OwlProfile::Dl));
}

#[test]
fn functional_object_property_detects_rl() {
    let ontology = ontology_with_profile_constructs(&[OwlConstruct::FunctionalObjectProperty]);
    let report = detect_profile(&ontology).expect("detect");
    assert_eq!(report.detected, Some(OwlProfile::Rl));
}

#[test]
fn inverse_object_properties_detects_rl() {
    let ontology = ontology_with_profile_constructs(&[OwlConstruct::InverseObjectProperties]);
    let report = detect_profile(&ontology).expect("detect");
    assert_eq!(report.detected, Some(OwlProfile::Rl));
}

#[test]
fn skipped_only_source_constructs_escalates_to_dl() {
    let mut meta = ParseMeta::default();
    meta.constructs.insert(OwlConstruct::ObjectUnionOf);
    meta.skipped_axiom_count = 1;
    let mut ontology = Ontology::new();
    ontology.set_parse_meta(meta);

    let report = detect_profile(&ontology).expect("detect");
    assert_eq!(report.detected, Some(OwlProfile::Dl));
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diag| diag.construct == "SkippedAxioms"),
        "expected skipped-only DL diagnostic, got {:?}",
        report.diagnostics
    );
}

#[test]
fn mixed_el_and_rl_forbidden_mapped_constructs_detect_dl_with_diagnostics() {
    let ontology = ontology_with_profile_constructs(&[
        OwlConstruct::SubClassOfExistential,
        OwlConstruct::InverseObjectProperties,
    ]);
    let report = detect_profile(&ontology).expect("detect");
    assert_eq!(report.detected, Some(OwlProfile::Dl));
    assert!(
        report.diagnostics.len() >= 2,
        "expected EL and RL violation diagnostics, got {:?}",
        report.diagnostics
    );
}
