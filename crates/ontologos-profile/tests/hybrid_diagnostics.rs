use ontologos_core::{Ontology, OwlConstruct, ParseMeta};
use ontologos_profile::{OwlProfile, detect_profile};

#[test]
fn source_only_dl_construct_reported_in_diagnostics_while_detected_el() {
    let mut meta = ParseMeta::default();
    meta.profile_constructs
        .insert(OwlConstruct::SubClassOfExistential);
    meta.constructs.insert(OwlConstruct::SubClassOfExistential);
    meta.constructs.insert(OwlConstruct::ObjectAllValuesFrom);

    let mut ontology = Ontology::new();
    ontology.set_parse_meta(meta);

    let report = detect_profile(&ontology).expect("detect");
    assert_eq!(report.detected, Some(OwlProfile::El));
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diag| diag.construct.contains("ObjectAllValuesFrom")),
        "expected ObjectAllValuesFrom diagnostic, got {:?}",
        report.diagnostics
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diag| diag.message.contains("observed in source")),
        "expected source-only diagnostic message"
    );
}
