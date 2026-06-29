use ontologos_dl::is_consistent;
use ontologos_parser::load_ontology;

fn wg502() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D502/premise.rdf",
    )
}

#[test]
fn wg502_consistency() {
    let ont = load_ontology(&wg502()).expect("load");
    let kb = is_consistent(&ont).expect("consistent");
    assert!(!kb, "502 should be inconsistent");
}

#[test]
fn wg502_after_501() {
    let p501 = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D501/premise.rdf",
    );
    let _ = load_ontology(&p501).expect("load 501");
    let ont = load_ontology(&wg502()).expect("load 502");
    let kb = is_consistent(&ont).expect("consistent");
    assert!(!kb, "502 after 501 should be inconsistent");
}
