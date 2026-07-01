//! Regression: tableau must not clear clashes after witness materialization (audit C1).

use ontologos_alc::{TableauSeed, tableau_is_consistent_with_seed};
use ontologos_core::{ClassExpr, DlAxiom, EntityKind, Ontology, RoleExpr};

/// KB with conflicting ∃ fillers under max-cardinality 1 — must be inconsistent.
#[test]
fn witness_materialization_clash_is_inconsistent() {
    let mut ontology = Ontology::new();
    let a = ontology
        .entity_id("http://ex.org/A", EntityKind::Class)
        .expect("a");
    let b = ontology
        .entity_id("http://ex.org/B", EntityKind::Class)
        .expect("b");
    let r = ontology
        .entity_id("http://ex.org/r", EntityKind::ObjectProperty)
        .expect("r");
    let store = ontology.dl_mut();
    let top = store.intern_ce(ClassExpr::Top);
    let b_ce = store.intern_ce(ClassExpr::Atomic(b));
    let not_b = store.intern_ce(ClassExpr::Not(b_ce));
    let role = RoleExpr::Atomic(r);
    let exists_r_b = store.intern_ce(ClassExpr::Some {
        property: role.clone(),
        filler: b_ce,
    });
    let exists_r_not_b = store.intern_ce(ClassExpr::Some {
        property: role.clone(),
        filler: not_b,
    });
    let max1_r = store.intern_ce(ClassExpr::MaxCardinality {
        n: 1,
        property: role,
        filler: None,
    });
    let a_ce = store.intern_ce(ClassExpr::Atomic(a));
    let a_and_max = store.intern_ce(ClassExpr::And(vec![
        a_ce, max1_r, exists_r_b, exists_r_not_b,
    ]));
    store.push_axiom(DlAxiom::SubClassOf {
        sub: top,
        sup: a_and_max,
    });

    let sat = tableau_is_consistent_with_seed(&ontology, &TableauSeed::default())
        .expect("sat check");
    assert!(
        !sat,
        "conflicting ∃ witnesses under max=1 must be unsatisfiable, not consistent"
    );
}
