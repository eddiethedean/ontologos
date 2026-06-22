use ontologos_dl::is_consistent;
use ontologos_parser::load_ontology;
use std::path::Path;

#[test]
fn wg_float_discrete_001_is_inconsistent() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/wg/Datatype-2DFloat-2DDiscrete-2D001/premise.rdf");
    let ont = load_ontology(&path).expect("load");
    assert!(
        ont.dl().axiom_count() > 0,
        "expected DL axioms from typed individual"
    );
    assert!(!is_consistent(&ont).expect("consistent"));
}
