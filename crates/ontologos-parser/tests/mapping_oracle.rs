//! Parser mapping oracle — family.owl mapped axiom kinds per supported-constructs guide.

use ontologos_core::Axiom;
use ontologos_parser::load_ontology;
use std::path::PathBuf;

#[test]
fn family_owl_maps_expected_axiom_kinds() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data/family.owl");
    assert!(
        path.is_file(),
        "missing family.owl — run ./benchmarks/scripts/download.sh"
    );
    let ontology = load_ontology(&path).expect("load");
    let mut subclass = 0;
    let mut inverse = 0;
    let mut subproperty = 0;
    for (_, axiom) in ontology.axioms().iter() {
        match axiom {
            Axiom::SubClassOf { .. } => subclass += 1,
            Axiom::InverseObjectProperties { .. } => inverse += 1,
            Axiom::SubObjectPropertyOf { .. } => subproperty += 1,
            _ => {}
        }
    }
    for axiom in ontology.dl().axioms() {
        if matches!(axiom, ontologos_core::DlAxiom::SubClassOf { .. }) {
            subclass += 1;
        }
    }
    assert!(
        subclass >= 5,
        "family must map SubClassOf axioms (core + DL), got {subclass}"
    );
    assert!(inverse >= 1, "family must map inverse property axioms");
    assert!(subproperty >= 1, "family must map subPropertyOf axioms");
    assert!(
        ontology.axiom_count() >= 50,
        "mapper axiom_count is not Protégé logical count but must reflect mapped output"
    );
}
