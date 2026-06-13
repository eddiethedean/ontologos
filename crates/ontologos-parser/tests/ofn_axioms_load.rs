//! HermiT OFN axiom fixtures must load without skip warnings.

use std::path::Path;

use ontologos_parser::load_ontology;

#[test]
fn hermit_ofn_axioms_load_without_skips() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/axioms");
    assert!(dir.is_dir(), "missing axiom dir: {}", dir.display());

    let mut loaded = 0usize;
    let mut failures = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("read axiom dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("ofn") {
            continue;
        }
        loaded += 1;
        let ontology = match load_ontology(&path) {
            Ok(o) => o,
            Err(e) => {
                failures.push(format!("{}: load error: {e}", path.display()));
                continue;
            }
        };
        let Some(meta) = ontology.parse_meta() else {
            failures.push(format!("{}: missing parse meta", path.display()));
            continue;
        };
        if meta.skipped_axiom_count > 0 {
            failures.push(format!(
                "{}: {} skipped axioms: {:?}",
                path.display(),
                meta.skipped_axiom_count,
                meta.warnings
            ));
        }
    }
    assert!(loaded > 0, "expected OFN axiom fixtures in {}", dir.display());
    assert!(failures.is_empty(), "OFN load failures:\n{}", failures.join("\n"));
}
