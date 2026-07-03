//! DL propagation of ALC tableau resource limits (moved from ontologos-alc).

use ontologos_alc::{Error, TableauSeed, tableau_is_consistent_with_seed};
use ontologos_core::{ClassExpr, DlAxiom, EntityKind, Ontology, RoleExpr};

fn long_existential_chain_ontology() -> Ontology {
    let mut ontology = Ontology::new();
    let top = ontology
        .entity_id("http://ex/#Thing", EntityKind::Class)
        .expect("top");
    let role = ontology
        .entity_id("http://ex/#r", EntityKind::ObjectProperty)
        .expect("role");
    let ind = ontology
        .entity_id("http://ex/#a", EntityKind::Individual)
        .expect("ind");
    let store = ontology.dl_mut();
    let filler = store.intern_ce(ClassExpr::Atomic(top));
    let mut chain = store.intern_ce(ClassExpr::Some {
        property: RoleExpr::Atomic(role),
        filler,
    });
    for _ in 0..300 {
        chain = store.intern_ce(ClassExpr::Some {
            property: RoleExpr::Atomic(role),
            filler: chain,
        });
    }
    store.push_axiom(DlAxiom::ClassAssertion {
        individual: ind,
        class: chain,
    });
    ontology
}

#[test]
fn dl_propagates_tableau_resource_limit() {
    let ontology = long_existential_chain_ontology();
    let alc_result = tableau_is_consistent_with_seed(&ontology, &TableauSeed::default());
    assert!(
        matches!(alc_result, Err(Error::ResourceLimit(_)) | Ok(false)),
        "ALC tableau must not report SAT under resource pressure: {alc_result:?}"
    );

    let result = ontologos_dl::check_consistency(&ontology, None);
    assert!(
        matches!(
            result,
            Ok(ontologos_core::ConsistencyResult {
                complete: false,
                ..
            }) | Err(ontologos_dl::Error::IncompleteReasoning(_))
                | Ok(ontologos_core::ConsistencyResult {
                    consistent: false,
                    complete: true
                })
        ),
        "DL must not map ResourceLimit to proved consistent: {result:?}"
    );
}
