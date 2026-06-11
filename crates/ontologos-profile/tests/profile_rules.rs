use ontologos_core::{Ontology, OwlConstruct, ParseMeta};
use ontologos_profile::{detect_profile, OwlProfile};

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
fn object_all_values_from_is_not_el_or_rl() {
    let ontology = ontology_with_profile_constructs(&[OwlConstruct::ObjectAllValuesFrom]);
    let report = detect_profile(&ontology).expect("detect");
    assert_ne!(report.detected, Some(OwlProfile::El));
    assert_ne!(report.detected, Some(OwlProfile::Rl));
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
        OwlConstruct::SymmetricObjectProperty,
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
fn ql_forbidden_construct_escalates_profile() {
    let ontology = ontology_with_profile_constructs(&[OwlConstruct::ObjectUnionOf]);
    let report = detect_profile(&ontology).expect("detect");
    assert_ne!(report.detected, Some(OwlProfile::Ql));
}
