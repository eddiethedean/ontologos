//! Regression tests for confirmed bugs tracked on GitHub.
//! Run ignored tests: `cargo test -p ontologos-parser -- --ignored`

use std::path::Path;

use ontologos_parser::load_ontology;

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// Parser should report entity kind mismatch, not "complex operands" skip.
#[test]
#[ignore = "bug #2: parser swallows EntityKindMismatch"]
fn class_assertion_kind_clash_surfaces_entity_kind_mismatch() {
    let ontology = load_ontology(&fixture("class_individual_kind_clash.ttl")).expect("load");
    let meta = ontology.parse_meta().expect("parse_meta");
    assert!(
        meta.warnings
            .iter()
            .any(|w| { w.contains("entity kind mismatch") || w.contains("EntityKindMismatch") }),
        "expected kind mismatch warning, got: {:?}",
        meta.warnings
    );
    assert!(
        !meta.warnings.iter().any(|w| w.contains("complex operands")),
        "should not mislabel kind clash as unmapped complex expression"
    );
}
