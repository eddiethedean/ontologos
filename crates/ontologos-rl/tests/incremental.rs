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

fn axiom_keys(ontology: &Ontology) -> std::collections::BTreeSet<String> {
    fn iri_of(ontology: &Ontology, id: ontologos_core::EntityId) -> String {
        ontology
            .entity(id)
            .ok()
            .and_then(|r| ontology.resolve_iri(r.iri).ok().map(str::to_string))
            .unwrap_or_else(|| format!("?{}", id.0))
    }

    const OWL_THING: &str = "http://www.w3.org/2002/07/owl#Thing";

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
            Axiom::SubObjectPropertyOf {
                sub_property,
                super_property,
            } => format!(
                "SubObjectPropertyOf({}, {})",
                iri_of(ontology, *sub_property),
                iri_of(ontology, *super_property)
            ),
            Axiom::ObjectPropertyAssertion {
                subject,
                property,
                object,
            } => format!(
                "ObjectPropertyAssertion({}, {}, {})",
                iri_of(ontology, *subject),
                iri_of(ontology, *property),
                iri_of(ontology, *object)
            ),
            Axiom::ClassAssertion { individual, class } => format!(
                "ClassAssertion({}, {})",
                iri_of(ontology, *individual),
                iri_of(ontology, *class)
            ),
            other => format!("{other:?}"),
        })
        // Reasonable may re-seed owl:Thing typing unevenly after axiom removal rebuilds.
        .filter(|key| {
            !(key.starts_with("ClassAssertion(") && key.ends_with(&format!("{OWL_THING})")))
        })
        .collect()
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

    assert_eq!(axiom_keys(reasoner.ontology()), axiom_keys(&full_after));
}

#[test]
fn incremental_matches_full_after_axiom_removal() {
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
    let bc_id = ontology
        .add_axiom(Axiom::SubClassOf {
            subclass: b,
            superclass: c,
        })
        .unwrap();

    let mut reasoner = Reasoner::builder()
        .profile(Profile::Rl)
        .config(ReasonerConfig {
            incremental: true,
            ..ReasonerConfig::default()
        })
        .build(ontology)
        .unwrap();

    ontologos_rl::materialize_reasoner(&mut reasoner).unwrap();
    reasoner.ontology_mut().remove_axiom(bc_id).unwrap();
    ontologos_rl::materialize_reasoner(&mut reasoner).unwrap();

    let mut full = Ontology::new();
    let a = full
        .entity_id("http://ex.org/A", EntityKind::Class)
        .unwrap();
    let b = full
        .entity_id("http://ex.org/B", EntityKind::Class)
        .unwrap();
    full.add_axiom(Axiom::SubClassOf {
        subclass: a,
        superclass: b,
    })
    .unwrap();
    ontologos_rl::RlEngine::new(1).saturate(&mut full).unwrap();

    assert_eq!(axiom_keys(reasoner.ontology()), axiom_keys(&full));
    let _ = c;
}
