use ontologos_core::{Axiom, EntityKind, Ontology, Profile, Reasoner, ReasonerConfig};

fn family_base() -> Ontology {
    Ontology::builder()
        .individual("http://ex.org/John")
        .unwrap()
        .individual("http://ex.org/Mary")
        .unwrap()
        .object_property("http://ex.org/hasParent")
        .unwrap()
        .object_property("http://ex.org/hasAncestor")
        .unwrap()
        .subproperty_of("http://ex.org/hasParent", "http://ex.org/hasAncestor")
        .unwrap()
        .object_property_assertion(
            "http://ex.org/John",
            "http://ex.org/hasParent",
            "http://ex.org/Mary",
        )
        .unwrap()
        .build()
        .unwrap()
}

#[test]
fn incremental_matches_full_after_axiom_addition() {
    let mut full = family_base();
    ontologos_rl::RlEngine::new(1).saturate(&mut full).unwrap();

    let mut reasoner = Reasoner::builder()
        .profile(Profile::Rl)
        .config(ReasonerConfig {
            incremental: true,
            ..ReasonerConfig::default()
        })
        .build(family_base())
        .unwrap();

    ontologos_rl::materialize_reasoner(&mut reasoner).unwrap();

    let ont = reasoner.ontology_mut();
    let j = ont
        .entity_id("http://ex.org/John", EntityKind::Individual)
        .unwrap();
    let a = ont
        .entity_id("http://ex.org/Alice", EntityKind::Individual)
        .unwrap();
    let has_parent = ont
        .entity_id("http://ex.org/hasParent", EntityKind::ObjectProperty)
        .unwrap();
    ont.add_axiom(Axiom::ObjectPropertyAssertion {
        subject: j,
        property: has_parent,
        object: a,
    })
    .unwrap();

    ontologos_rl::materialize_reasoner(&mut reasoner).unwrap();

    let mut full_after = family_base();
    let j = full_after
        .entity_id("http://ex.org/John", EntityKind::Individual)
        .unwrap();
    let a = full_after
        .entity_id("http://ex.org/Alice", EntityKind::Individual)
        .unwrap();
    let has_parent = full_after
        .entity_id("http://ex.org/hasParent", EntityKind::ObjectProperty)
        .unwrap();
    full_after
        .add_axiom(Axiom::ObjectPropertyAssertion {
            subject: j,
            property: has_parent,
            object: a,
        })
        .unwrap();
    ontologos_rl::RlEngine::new(1)
        .saturate(&mut full_after)
        .unwrap();

    assert_eq!(reasoner.ontology().axiom_count(), full_after.axiom_count());
}
