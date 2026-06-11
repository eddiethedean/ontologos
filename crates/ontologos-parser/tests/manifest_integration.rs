use std::path::{Path, PathBuf};

use ontologos_parser::load_ontology;
use ontologos_profile::{detect_profile, OwlProfile};

struct ManifestEntry {
    name: &'static str,
    local_path: &'static str,
    expected_profile: OwlProfile,
    axiom_count_approx: usize,
}

const ENTRIES: &[ManifestEntry] = &[
    ManifestEntry {
        name: "pizza",
        local_path: "benchmarks/data/pizza.owl",
        expected_profile: OwlProfile::El,
        axiom_count_approx: 1056,
    },
    ManifestEntry {
        name: "family",
        local_path: "benchmarks/data/family.owl",
        expected_profile: OwlProfile::Rl,
        axiom_count_approx: 57,
    },
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

#[test]
fn manifest_corpus_load_and_profile() {
    let root = repo_root();
    for entry in ENTRIES {
        let path = root.join(entry.local_path);
        assert!(
            path.exists(),
            "missing benchmark corpus {} at {} (run ./benchmarks/scripts/download.sh)",
            entry.name,
            path.display()
        );

        let ontology = load_ontology(&path).unwrap_or_else(|e| {
            panic!("load {} failed: {e}", entry.name);
        });

        let tolerance = (entry.axiom_count_approx as f64 * 0.10).ceil() as usize;
        let count = ontology.axiom_count();
        let low = entry.axiom_count_approx.saturating_sub(tolerance);
        let high = entry.axiom_count_approx + tolerance;
        assert!(
            (low..=high).contains(&count),
            "{} axiom count {count} outside {} ±10%",
            entry.name,
            entry.axiom_count_approx
        );

        let report = detect_profile(&ontology).expect("profile");
        assert_eq!(
            report.detected,
            Some(entry.expected_profile),
            "{} profile mismatch",
            entry.name
        );
    }
}
