//! Optional stress tests for large benchmark corpora.
//!
//! Run with: `cargo test -p ontologos-parser --test corpus_stress -- --ignored`

use std::path::{Path, PathBuf};

use ontologos_parser::load_ontology;
use ontologos_profile::{detect_profile, OwlProfile};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn corpus_path(relative: &str) -> PathBuf {
    repo_root().join(relative)
}

fn load_if_present(relative: &str) -> Option<ontologos_core::Ontology> {
    let path = corpus_path(relative);
    if !path.exists() {
        eprintln!("skip: missing {}", path.display());
        return None;
    }
    Some(load_ontology(&path).unwrap_or_else(|e| {
        panic!("load {} failed: {e}", path.display());
    }))
}

#[test]
#[ignore = "requires benchmarks/data/galen.owl (manual download)"]
fn galen_loads_and_is_el() {
    let ontology = load_if_present("benchmarks/data/galen.owl").expect("galen");
    let report = detect_profile(&ontology).expect("profile");
    assert_eq!(report.detected, Some(OwlProfile::El));
    assert!(ontology.axiom_count() > 1_000);
}

#[test]
#[ignore = "requires benchmarks/data/go-subset.owl (ROBOT trim of GO)"]
fn go_subset_loads_and_is_el() {
    let ontology = load_if_present("benchmarks/data/go-subset.owl").expect("go-subset");
    let report = detect_profile(&ontology).expect("profile");
    assert_eq!(report.detected, Some(OwlProfile::El));
    assert!(ontology.axiom_count() > 1_000);
}

#[test]
#[ignore = "requires benchmarks/data/go.owl (large; manual download)"]
fn go_full_loads_without_panic() {
    let ontology = load_if_present("benchmarks/data/go.owl").expect("go");
    assert!(ontology.axiom_count() > 10_000);
}
