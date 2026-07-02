//! Verify WG wine import merge loads both consistent ontologies.

use ontologos_parser::load_ontology;
use std::path::PathBuf;

#[test]
fn misc_001_merge_includes_consistent001_and_002() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2Dmiscellaneous-2D001/premise.rdf",
    );
    let ont = load_ontology(&path).expect("load");
    let mut has_001 = false;
    let mut has_002 = false;
    for (_, record) in ont.entities().iter() {
        let Ok(iri) = ont.resolve_iri(record.iri) else {
            continue;
        };
        if iri.contains("miscellaneous/consistent001") {
            has_001 = true;
        }
        if iri.contains("miscellaneous/consistent002") {
            has_002 = true;
        }
    }
    assert!(
        has_001,
        "expected consistent001 entities from owl:imports merge"
    );
    assert!(has_002, "expected consistent002 entities in premise");
}
