//! Parser + DL consistency for OWL WG disjointWith-010 (moved from ontologos-parser).

use ontologos_dl::is_consistent;
use ontologos_parser::load_ontology;

#[test]
fn disjoint_with_010_premise_is_inconsistent() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DdisjointWith-2D010/premise.rdf");
    let loaded = load_ontology(&path).expect("load");
    assert!(
        loaded.dl().axiom_count() > 0 || !loaded.axioms().is_empty(),
        "loaded ontology should have axioms"
    );
    assert!(
        !is_consistent(&loaded).expect("check"),
        "disjointWith-010 should be inconsistent"
    );
}
