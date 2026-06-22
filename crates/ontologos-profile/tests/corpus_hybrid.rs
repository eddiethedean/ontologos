//! Real-corpus hybrid routing smoke (Phase 1.5).

use std::path::{Path, PathBuf};

use ontologos_parser::load_ontology;
use ontologos_profile::{classify_hybrid, detect_profile, OwlProfile};

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
fn pizza_detects_dl_and_hybrid_partitions() {
    let ontology = load_corpus("benchmarks/data/pizza.owl").expect("pizza.owl");
    let report = detect_profile(&ontology).expect("profile");
    assert_eq!(report.detected, Some(OwlProfile::Dl));
    let hybrid = classify_hybrid(&ontology).expect("hybrid");
    assert!(!hybrid.modules.is_empty());
}

#[test]
fn go_subset_hybrid_when_present() {
    let Some(ontology) = load_corpus("benchmarks/data/go-subset.owl") else {
        return;
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
