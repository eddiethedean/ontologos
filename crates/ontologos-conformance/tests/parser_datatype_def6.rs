//! Parser DL datatype mapping checks that require ontologos-dl (moved from ontologos-parser).

use ontologos_dl::{LiteralIndex, LiteralValue, is_datatype_consistent};
use ontologos_parser::load_ontology;

#[test]
fn maps_datatype_def6_before_property_range() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_datatypestest_testdatatypedef6.ofn",
    );
    let ont = load_ontology(&path).unwrap();
    let store = ont.dl();
    let mut range_id = None;
    for ax in store.axioms() {
        if let ontologos_core::DlAxiom::DataPropertyRange { range, .. } = ax {
            range_id = Some(*range);
        }
    }
    let range_id = range_id.expect("range");
    let idx = LiteralIndex::from_store(store);
    let lit = LiteralValue {
        lexical: "16".into(),
        datatype: ont
            .lookup_entity("http://www.w3.org/2001/XMLSchema#integer")
            .unwrap(),
    };
    assert!(
        idx.satisfies_with_ontology(&lit, &ont, range_id),
        "de={:?}",
        store.de(range_id)
    );
    assert!(is_datatype_consistent(&ont));
}
