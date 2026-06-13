//! Hybrid module routing tests.

use ontologos_core::Ontology;
use ontologos_profile::{classify_hybrid, engine_for_profile, OwlProfile};

#[test]
fn hybrid_report_single_module() {
    let ontology = Ontology::default();
    let report = classify_hybrid(&ontology).expect("hybrid");
    assert_eq!(report.modules.len(), 1);
}

#[test]
fn engine_for_el_profile() {
    assert_eq!(engine_for_profile(OwlProfile::El), "el");
    assert_eq!(engine_for_profile(OwlProfile::Dl), "dl");
}
