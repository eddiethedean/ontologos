//! Bridge incremental session correctness tests.

use ontologos_bridge::{MergeLimits, ReasonableSession, materialize_with_session};
use ontologos_core::{Axiom, AxiomId, EntityKind, Ontology, Profile};

fn class_chain() -> (
    Ontology,
    ontologos_core::EntityId,
    ontologos_core::EntityId,
    ontologos_core::EntityId,
    AxiomId,
) {
    let mut ontology = Ontology::new();
    let a = ontology
        .entity_id("http://ex.org/A", EntityKind::Class)
        .unwrap();
    let b = ontology
        .entity_id("http://ex.org/B", EntityKind::Class)
        .unwrap();
    let c = ontology
        .entity_id("http://ex.org/C", EntityKind::Class)
        .unwrap();
    ontology
        .add_axiom(Axiom::SubClassOf {
            subclass: a,
            superclass: b,
        })
        .unwrap();
    let bc = ontology
        .add_axiom(Axiom::SubClassOf {
            subclass: b,
            superclass: c,
        })
        .unwrap();
    (ontology, a, b, c, bc)
}

fn axiom_keys(ontology: &Ontology) -> std::collections::BTreeSet<String> {
    fn iri_of(ontology: &Ontology, id: ontologos_core::EntityId) -> String {
        ontology
            .entity(id)
            .ok()
            .and_then(|r| ontology.resolve_iri(r.iri).ok().map(str::to_string))
            .unwrap_or_else(|| format!("?{}", id.0))
    }

    ontology
        .axioms()
        .iter()
        .map(|(_, axiom)| match axiom {
            Axiom::SubClassOf {
                subclass,
                superclass,
            } => format!(
                "SubClassOf({}, {})",
                iri_of(ontology, *subclass),
                iri_of(ontology, *superclass)
            ),
            Axiom::ClassAssertion { individual, class } => format!(
                "ClassAssertion({}, {})",
                iri_of(ontology, *individual),
                iri_of(ontology, *class)
            ),
            other => format!("{other:?}"),
        })
        .collect()
}

#[test]
fn removal_strips_stale_inferred_subsumption() {
    let (mut ontology, _a, _b, _c, bc_id) = class_chain();

    let (outcome, session) = materialize_with_session(
        &mut ontology,
        ReasonableSession::new_for_profile(Profile::Rl),
        false,
        MergeLimits::default(),
    )
    .expect("initial materialize");
    assert!(outcome.merge.inferred_axioms > 0);

    ontology.remove_axiom(bc_id).unwrap();

    let (outcome, _session) =
        materialize_with_session(&mut ontology, session, true, MergeLimits::default())
            .expect("rematerialize after removal");
    assert!(outcome.full_rebuild);

    let keys = axiom_keys(&ontology);
    let has_ac = keys.iter().any(|k| {
        k.contains("SubClassOf") && k.contains("A") && k.contains("C") && !k.contains("B")
    });
    assert!(!has_ac, "stale A sub C should be gone: {keys:?}");
}

#[test]
fn incremental_add_matches_full_materialize() {
    let mut full = class_chain().0;
    materialize_with_session(
        &mut full,
        ReasonableSession::new_for_profile(Profile::Rl),
        false,
        MergeLimits::default(),
    )
    .unwrap();

    let mut ontology = class_chain().0;
    let d = ontology
        .entity_id("http://ex.org/D", EntityKind::Class)
        .unwrap();
    let c = ontology
        .entity_id("http://ex.org/C", EntityKind::Class)
        .unwrap();

    let (_, session) = materialize_with_session(
        &mut ontology,
        ReasonableSession::new_for_profile(Profile::Rl),
        false,
        MergeLimits::default(),
    )
    .unwrap();

    ontology
        .add_axiom(Axiom::SubClassOf {
            subclass: c,
            superclass: d,
        })
        .unwrap();

    materialize_with_session(&mut ontology, session, true, MergeLimits::default()).unwrap();

    let c_full = full
        .entity_id("http://ex.org/C", EntityKind::Class)
        .unwrap();
    let d_full = full
        .entity_id("http://ex.org/D", EntityKind::Class)
        .unwrap();
    full.add_axiom(Axiom::SubClassOf {
        subclass: c_full,
        superclass: d_full,
    })
    .unwrap();
    materialize_with_session(
        &mut full,
        ReasonableSession::new_for_profile(Profile::Rl),
        false,
        MergeLimits::default(),
    )
    .unwrap();

    assert_eq!(axiom_keys(&ontology), axiom_keys(&full));
}
