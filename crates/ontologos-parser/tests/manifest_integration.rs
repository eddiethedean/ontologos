use std::path::{Path, PathBuf};

use ontologos_core::OwlConstruct;
use ontologos_parser::load_ontology;
use ontologos_profile::{OwlProfile, detect_profile};

struct ManifestEntry {
    name: &'static str,
    local_path: &'static str,
    expected_profile: OwlProfile,
    /// Expected mapped axiom count from the parser mapper (not raw OWL logical axiom count).
    axiom_count_approx: usize,
}

const ENTRIES: &[ManifestEntry] = &[
    ManifestEntry {
        name: "pizza",
        local_path: "benchmarks/data/pizza.owl",
        expected_profile: OwlProfile::Dl,
        // Mapper output count; see benchmarks/manifest.toml and benchmarks/README.md.
        axiom_count_approx: 669,
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
            "{} axiom count {count} outside {} ±10% (mapper output, not raw OWL logical count)",
            entry.name,
            entry.axiom_count_approx
        );

        let meta = ontology
            .parse_meta()
            .unwrap_or_else(|| panic!("{} missing parse_meta", entry.name));
        assert_eq!(
            meta.mapped_axiom_count, count,
            "{} mapped_axiom_count should match axiom_count",
            entry.name
        );
        assert_eq!(
            meta.mapped_axiom_count + meta.skipped_axiom_count,
            meta.logical_axiom_count,
            "{} mapped + skipped should equal logical",
            entry.name
        );

        let report = detect_profile(&ontology).expect("profile");
        assert_eq!(
            report.detected,
            Some(entry.expected_profile),
            "{} profile mismatch",
            entry.name
        );

        match entry.name {
            "pizza" => {
                assert!(
                    meta.constructs.contains(&OwlConstruct::ObjectAllValuesFrom)
                        || meta.constructs.contains(&OwlConstruct::ObjectUnionOf),
                    "pizza source should contain DL constructs in parse_meta.constructs"
                );
                assert!(
                    !report.diagnostics.is_empty(),
                    "pizza classified as Dl should report why mapped constructs rule out EL/RL"
                );
            }
            "family" => {
                assert!(
                    meta.profile_constructs
                        .contains(&OwlConstruct::SymmetricObjectProperty)
                        || meta
                            .profile_constructs
                            .contains(&OwlConstruct::TransitiveObjectProperty)
                        || meta
                            .profile_constructs
                            .contains(&OwlConstruct::ReflexiveObjectProperty),
                    "family profile_constructs should contain RL markers"
                );
                for diag in &report.diagnostics {
                    assert!(
                        diag.message.contains("observed in source"),
                        "family diagnostics should be source-only under hybrid contract: {diag:?}"
                    );
                }
            }
            _ => {}
        }
    }
}
