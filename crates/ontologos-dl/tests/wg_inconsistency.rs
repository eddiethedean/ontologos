use ontologos_parser::load_ontology;
use std::path::PathBuf;

#[test]
fn footnote_is_inconsistent() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/wg/Footnote-2Dnot-2Dabout-2Dself/premise.rdf");
    let ont = load_ontology(&path).expect("load");
    let c = ontologos_dl::is_consistent(&ont).expect("check");
    assert!(!c, "expected inconsistent, got {c}");
}

#[test]
fn top_object_property_is_inconsistent() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/New-2DFeature-2DTopObjectProperty-2D001/premise.rdf",
    );
    let ont = load_ontology(&path).expect("load");
    let c = ontologos_dl::is_consistent(&ont).expect("check");
    assert!(!c, "expected inconsistent, got {c}");
}
