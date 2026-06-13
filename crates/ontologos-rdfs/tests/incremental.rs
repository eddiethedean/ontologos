use ontologos_core::{Axiom, EntityKind, Ontology, Profile, Reasoner, ReasonerConfig};

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
            other => format!("{other:?}"),
        })
        .collect()
}

#[test]
fn incremental_matches_full_after_axiom_addition() {
    let mut full = Ontology::builder()
        .class("http://ex.org/Dog")
        .unwrap()
        .class("http://ex.org/Animal")
        .unwrap()
        .subclass_of("http://ex.org/Dog", "http://ex.org/Animal")
        .unwrap()
        .build()
        .unwrap();

    ontologos_rdfs::RdfsEngine::new()
        .materialize(&mut full)
        .unwrap();

    let mut reasoner = Reasoner::builder()
        .profile(Profile::Rdfs)
        .config(ReasonerConfig {
            incremental: true,
            ..ReasonerConfig::default()
        })
        .build(
            Ontology::builder()
                .class("http://ex.org/Dog")
                .unwrap()
                .class("http://ex.org/Animal")
                .unwrap()
                .subclass_of("http://ex.org/Dog", "http://ex.org/Animal")
                .unwrap()
                .build()
                .unwrap(),
        )
        .unwrap();

    ontologos_rdfs::materialize_reasoner(&mut reasoner).unwrap();

    let cat = reasoner
        .ontology_mut()
        .entity_id("http://ex.org/Cat", EntityKind::Class)
        .unwrap();
    let animal = reasoner
        .ontology_mut()
        .entity_id("http://ex.org/Animal", EntityKind::Class)
        .unwrap();
    reasoner
        .ontology_mut()
        .add_axiom(Axiom::SubClassOf {
            subclass: cat,
            superclass: animal,
        })
        .unwrap();

    ontologos_rdfs::materialize_reasoner(&mut reasoner).unwrap();

    let cat_full = full
        .entity_id("http://ex.org/Cat", EntityKind::Class)
        .unwrap();
    let animal_full = full
        .entity_id("http://ex.org/Animal", EntityKind::Class)
        .unwrap();
    full.add_axiom(Axiom::SubClassOf {
        subclass: cat_full,
        superclass: animal_full,
    })
    .unwrap();
    ontologos_rdfs::RdfsEngine::new()
        .materialize(&mut full)
        .unwrap();

    assert_eq!(axiom_keys(reasoner.ontology()), axiom_keys(&full));
}
