use std::path::Path;

use ontologos_parser::load_ontology;
use ontologos_profile::{detect_profile, OwlProfile};

#[test]
fn family_detects_rl_profile() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data/family.owl");
    if !path.exists() {
        return;
    }
    let ontology = load_ontology(&path).expect("load family");
    eprintln!("family axioms={}", ontology.axiom_count());
    if let Some(meta) = ontology.parse_meta() {
        eprintln!("profile_constructs={:?}", meta.profile_constructs);
    }
    let report = detect_profile(&ontology).expect("detect");
    assert_eq!(report.detected, Some(OwlProfile::Rl));
}
