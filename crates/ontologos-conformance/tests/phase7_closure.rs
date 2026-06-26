//! Phase 7 exit gate — Tier C DL taxonomy goldens and HermiT cross-check harness.

use std::path::PathBuf;

fn benchmarks_data() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data")
}

fn benchmarks_scripts() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/scripts")
}

#[test]
fn phase7_dl_golden_vendored() {
    let golden = benchmarks_data().join("dl-taxonomy-golden.json");
    assert!(golden.is_file(), "missing {}", golden.display());
    let text = std::fs::read_to_string(&golden).expect("read golden");
    let doc: serde_json::Value = serde_json::from_str(&text).expect("parse golden");
    let family = doc
        .get("corpora")
        .and_then(|c| c.get("family.owl"))
        .expect("family.owl corpus in golden");
    assert_eq!(
        family.get("profile").and_then(|v| v.as_str()),
        Some("dl")
    );
    assert!(
        family.get("subsumption_count").and_then(|v| v.as_u64()).unwrap_or(0) > 0,
        "family.owl golden should list subsumptions"
    );
}

#[test]
fn phase7_family_dl_smoke() {
    use ontologos_dl::classify;
    use ontologos_parser::load_ontology;

    let path = benchmarks_data().join("family.owl");
    assert!(path.is_file(), "missing {}", path.display());

    let golden_text =
        std::fs::read_to_string(benchmarks_data().join("dl-taxonomy-golden.json")).unwrap();
    let doc: serde_json::Value = serde_json::from_str(&golden_text).unwrap();
    let expected = doc["corpora"]["family.owl"]["subsumption_count"]
        .as_u64()
        .expect("golden subsumption_count");

    let ontology = load_ontology(&path).expect("load family.owl");
    let taxonomy = classify(&ontology).expect("dl classify family.owl");
    assert_eq!(
        taxonomy.subsumption_count() as u64,
        expected,
        "family DL subsumption_count should match golden"
    );
}

#[test]
fn phase7_scripts_present() {
    for script in [
        "compare-dl-taxonomy.sh",
        "compare-dl-hermit-crosscheck.sh",
        "download-hermit-jar.sh",
        "compare-tier-c-gate.sh",
        "benchmark-dl-perf.sh",
    ] {
        let path = benchmarks_scripts().join(script);
        assert!(path.is_file(), "missing script {}", path.display());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path)
                .expect("stat script")
                .permissions()
                .mode();
            assert!(mode & 0o111 != 0, "{} should be executable", path.display());
        }
    }
}
