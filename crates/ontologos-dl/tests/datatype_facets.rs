//! XSD facet boundary tests.

use ontologos_core::{DataExpr, DlStore, EntityId};
use ontologos_dl::LiteralIndex;

fn xsd_decimal() -> EntityId {
    EntityId(1)
}

#[test]
fn exclusive_numeric_bounds() {
    let mut store = DlStore::default();
    let decimal = xsd_decimal();
    let base = store.intern_de(DataExpr::Datatype(decimal));
    let max_ex = store.intern_de(DataExpr::Facet {
        base,
        facet_iri: "http://www.w3.org/2001/XMLSchema#maxExclusive".into(),
        value: "10".into(),
    });
    let min_ex = store.intern_de(DataExpr::Facet {
        base,
        facet_iri: "http://www.w3.org/2001/XMLSchema#minExclusive".into(),
        value: "0".into(),
    });

    let idx = LiteralIndex::default();
    let lit = |lex: &str| ontologos_dl::LiteralValue {
        lexical: lex.into(),
        datatype: decimal,
    };

    assert!(idx.satisfies(&lit("5"), &store, max_ex));
    assert!(!idx.satisfies(&lit("10"), &store, max_ex));
    assert!(idx.satisfies(&lit("1"), &store, min_ex));
    assert!(!idx.satisfies(&lit("0"), &store, min_ex));
}

#[test]
fn inclusive_numeric_bounds() {
    let mut store = DlStore::default();
    let decimal = xsd_decimal();
    let base = store.intern_de(DataExpr::Datatype(decimal));
    let max_in = store.intern_de(DataExpr::Facet {
        base,
        facet_iri: "http://www.w3.org/2001/XMLSchema#maxInclusive".into(),
        value: "10".into(),
    });
    let min_in = store.intern_de(DataExpr::Facet {
        base,
        facet_iri: "http://www.w3.org/2001/XMLSchema#minInclusive".into(),
        value: "0".into(),
    });

    let idx = LiteralIndex::default();
    let lit = |lex: &str| ontologos_dl::LiteralValue {
        lexical: lex.into(),
        datatype: decimal,
    };

    assert!(idx.satisfies(&lit("10"), &store, max_in));
    assert!(idx.satisfies(&lit("0"), &store, min_in));
}

#[test]
fn unknown_facet_fails_closed() {
    let mut store = DlStore::default();
    let decimal = xsd_decimal();
    let base = store.intern_de(DataExpr::Datatype(decimal));
    let unknown = store.intern_de(DataExpr::Facet {
        base,
        facet_iri: "http://example.org/unknownFacet".into(),
        value: "1".into(),
    });
    let idx = LiteralIndex::default();
    let lit = ontologos_dl::LiteralValue {
        lexical: "1".into(),
        datatype: decimal,
    };
    assert!(!idx.satisfies(&lit, &store, unknown));
}
