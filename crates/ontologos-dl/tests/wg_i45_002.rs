use ontologos_dl::is_consistent;
use ontologos_parser::load_ontology;
use std::path::Path;

#[test]
fn i45_002_premise_is_inconsistent() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2DI4.5-2D002/premise.rdf");
    let ont = load_ontology(&path).expect("load");
    let consistent = is_consistent(&ont).expect("check");
    eprintln!("I4.5-002 consistent={consistent}");
    assert!(
        !consistent,
        "Kinnock/EuroMP/inverseOf case should be inconsistent"
    );
}
