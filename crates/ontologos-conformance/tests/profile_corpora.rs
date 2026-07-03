//! Profile detection and hybrid routing on benchmark corpora (moved from ontologos-profile).

use std::path::{Path, PathBuf};

use ontologos_parser::load_ontology;
use ontologos_profile::{OwlProfile, classify_hybrid, detect_profile, scanner::scan_constructs};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn load_corpus(relative: &str) -> Option<ontologos_core::Ontology> {
    let path = repo_root().join(relative);
    if !path.is_file() {
        return None;
    }
    Some(load_ontology(&path).expect("load corpus"))
}

#[test]
fn datatype_fixture_detected_as_dl_not_rl() {
    let path = repo_root().join(
        "benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testallvaluesfrominteger1.ofn",
    );
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

#[test]
fn pizza_detects_dl_and_hybrid_partitions() {
    let ontology = load_corpus("benchmarks/data/pizza.owl").expect("pizza.owl");
    let report = detect_profile(&ontology).expect("profile");
    assert_eq!(report.detected, Some(OwlProfile::Dl));
    let hybrid = classify_hybrid(&ontology).expect("hybrid");
    assert!(!hybrid.modules.is_empty());
}

#[test]
#[ignore = "optional corpus — run ./benchmarks/scripts/download.sh for go-subset.owl"]
fn go_subset_hybrid_when_present() {
    let Some(ontology) = load_corpus("benchmarks/data/go-subset.owl") else {
        panic!("go-subset.owl missing — run ./benchmarks/scripts/download.sh or ignore this test");
    };
    let report = detect_profile(&ontology).expect("profile");
    assert!(
        matches!(report.detected, Some(OwlProfile::El) | Some(OwlProfile::Ql)),
        "go-subset profile: {:?}",
        report.detected
    );
    let hybrid = classify_hybrid(&ontology).expect("hybrid");
    assert!(!hybrid.modules.is_empty());
}

#[test]
fn family_dl_profile_detected_or_rl() {
    let ontology = load_corpus("benchmarks/data/family.owl").expect("family");
    let report = detect_profile(&ontology).expect("profile");
    assert!(
        matches!(report.detected, Some(OwlProfile::Rl) | Some(OwlProfile::Dl)),
        "family corpus profile: {:?}",
        report.detected
    );
}

#[test]
#[ignore = "optional corpus — run ./benchmarks/scripts/download.sh for galen.owl"]
fn galen_hybrid_el_module() {
    let Some(ontology) = load_corpus("benchmarks/data/galen.owl") else {
        panic!("galen.owl missing — run ./benchmarks/scripts/download.sh or ignore this test");
    };
    let hybrid = classify_hybrid(&ontology).expect("hybrid");
    assert!(
        !hybrid.modules.is_empty(),
        "GALEN should partition into modules"
    );
    let el_modules: Vec<_> = hybrid
        .modules
        .iter()
        .filter(|m| m.profile == OwlProfile::El || m.profile == OwlProfile::Ql)
        .collect();
    assert!(
        !el_modules.is_empty(),
        "GALEN should have at least one EL/QL module"
    );
    let total_axioms: usize = hybrid.modules.iter().map(|m| m.axiom_ids.len()).sum();
    let dl_axiom_count: usize = hybrid
        .modules
        .iter()
        .filter(|m| m.profile == OwlProfile::Dl)
        .map(|m| m.axiom_ids.len())
        .sum();
    assert!(
        dl_axiom_count < total_axioms,
        "GALEN should not delegate entire ontology to DL when EL modules exist"
    );
}
