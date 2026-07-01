use ontologos_dl::is_consistent;
use ontologos_parser::load_ontology;
use std::path::PathBuf;

fn wg(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit")
        .join(rel)
}

#[test]
fn keys_002_premise_is_inconsistent() {
    let ont = load_ontology(&wg("wg/New-2DFeature-2DKeys-2D002/premise.rdf")).expect("load");
    assert!(
        !is_consistent(&ont).expect("consistency"),
        "Keys-002 premise should be inconsistent"
    );
}

#[test]
fn npa_dat_premise_is_inconsistent() {
    let ont = load_ontology(&wg("wg/Rdfbased-2Dsem-2Dnpa-2Ddat-2Dfw/premise.rdf")).expect("load");
    assert!(
        !is_consistent(&ont).expect("consistency"),
        "npa-dat-fw premise should be inconsistent"
    );
}

#[test]
fn i55_003_premise_is_inconsistent() {
    let ont = load_ontology(&wg("wg/TestCase-3AWebOnt-2DI5.5-2D003/premise.rdf")).expect("load");
    assert!(
        !is_consistent(&ont).expect("consistency"),
        "I5.5-003 premise should be inconsistent"
    );
}
