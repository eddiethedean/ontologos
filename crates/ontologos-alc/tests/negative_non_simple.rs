use ontologos_alc::tableau_is_consistent;
use ontologos_parser::load_ontology;
use std::path::PathBuf;

#[test]
fn negative_object_property_with_non_simple_role() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_reasonertest_testnegativeobjectpropertyassertionwithnonsimple.ofn",
    );
    let ont = load_ontology(&path).expect("load");
    let consistent = tableau_is_consistent(&ont).expect("check");
    assert!(!consistent, "expected inconsistent, got {consistent}");
}
