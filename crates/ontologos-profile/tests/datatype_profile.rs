use ontologos_parser::load_ontology;
use ontologos_profile::{OwlProfile, detect_profile, scanner::scan_constructs};
use std::path::PathBuf;

#[test]
fn datatype_fixture_detected_as_dl_not_rl() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testallvaluesfrominteger1.ofn");
    assert!(
        path.is_file(),
        "missing HermiT datatype fixture at {} (vendored with benchmarks/data/hermit)",
        path.display()
    );
    let ontology = load_ontology(&path).expect("load");
    let report = detect_profile(&ontology).expect("detect");
    let constructs = scan_constructs(&ontology);
    eprintln!("detected={:?} constructs={constructs:?}", report.detected);
    if let Some(meta) = ontology.parse_meta() {
        eprintln!("profile_constructs={:?}", meta.profile_constructs);
    }
    assert_ne!(report.detected, Some(OwlProfile::Rl));
}
