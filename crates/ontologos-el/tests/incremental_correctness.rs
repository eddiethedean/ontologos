use ontologos_core::{Axiom, EntityKind, Ontology, Profile, Reasoner, ReasonerConfig, Taxonomy};

fn class(ontology: &mut Ontology, iri: &str) -> ontologos_core::EntityId {
    ontology.entity_id(iri, EntityKind::Class).expect("class")
}

fn classify_full(ontology: &Ontology) -> Taxonomy {
    ontologos_el::ElClassifier::new()
        .classify(ontology)
        .expect("full classify")
}

fn classify_incremental(reasoner: &mut Reasoner) -> Taxonomy {
    ontologos_el::classify_reasoner(reasoner).expect("incremental classify")
}

fn assert_taxonomy_eq(a: &Taxonomy, b: &Taxonomy) {
    assert_eq!(a, b, "taxonomy mismatch");
}

#[test]
fn incremental_matches_full_transitive_chain_extension() {
    let mut ontology = Ontology::new();
    let a = class(&mut ontology, "http://ex.org/A");
    let b = class(&mut ontology, "http://ex.org/B");
    let c = class(&mut ontology, "http://ex.org/C");
    ontology
        .add_axiom(Axiom::SubClassOf {
            subclass: a,
            superclass: b,
        })
        .unwrap();
    ontology
        .add_axiom(Axiom::SubClassOf {
            subclass: b,
            superclass: c,
        })
        .unwrap();

    let full_before = classify_full(&ontology);

    let mut reasoner = Reasoner::builder()
        .profile(Profile::El)
        .config(ReasonerConfig {
            incremental: true,
            ..ReasonerConfig::default()
        })
        .build(ontology)
        .unwrap();

    let incr_before = classify_incremental(&mut reasoner);
    assert_taxonomy_eq(&full_before, &incr_before);

    let d = reasoner
        .ontology_mut()
        .entity_id("http://ex.org/D", EntityKind::Class)
        .unwrap();
    reasoner
        .ontology_mut()
        .add_axiom(Axiom::SubClassOf {
            subclass: c,
            superclass: d,
        })
        .unwrap();

    let full_after = classify_full(reasoner.ontology());
    let incr_after = classify_incremental(&mut reasoner);
    assert_taxonomy_eq(&full_after, &incr_after);
    assert!(incr_after.is_subsumed(a, d));
}

#[test]
fn incremental_matches_full_equivalence_edit() {
    let mut ontology = Ontology::new();
    let a = class(&mut ontology, "http://ex.org/A");
    let b = class(&mut ontology, "http://ex.org/B");
    let c = class(&mut ontology, "http://ex.org/C");
    ontology
        .add_axiom(Axiom::SubClassOf {
            subclass: a,
            superclass: c,
        })
        .unwrap();

    let mut reasoner = Reasoner::builder()
        .profile(Profile::El)
        .config(ReasonerConfig {
            incremental: true,
            ..ReasonerConfig::default()
        })
        .build(ontology)
        .unwrap();

    classify_incremental(&mut reasoner);

    reasoner
        .ontology_mut()
        .add_axiom(Axiom::EquivalentClasses(vec![a, b]))
        .unwrap();

    let full_after = classify_full(reasoner.ontology());
    let incr_after = classify_incremental(&mut reasoner);
    assert_taxonomy_eq(&full_after, &incr_after);
}

#[test]
fn ten_axiom_batch_matches_full() {
    let mut ontology = Ontology::new();
    let mut ids = Vec::new();
    for i in 0..11 {
        ids.push(class(&mut ontology, &format!("http://ex.org/C{i}")));
    }
    for i in 0..10 {
        ontology
            .add_axiom(Axiom::SubClassOf {
                subclass: ids[i],
                superclass: ids[i + 1],
            })
            .unwrap();
    }

    let mut reasoner = Reasoner::builder()
        .profile(Profile::El)
        .config(ReasonerConfig {
            incremental: true,
            ..ReasonerConfig::default()
        })
        .build(ontology)
        .unwrap();

    classify_incremental(&mut reasoner);

    for i in 0..10 {
        let ont = reasoner.ontology_mut();
        let x = ont
            .entity_id(&format!("http://ex.org/X{i}"), EntityKind::Class)
            .unwrap();
        let target = ont
            .entity_id(&format!("http://ex.org/C{i}"), EntityKind::Class)
            .unwrap();
        ont.add_axiom(Axiom::SubClassOf {
            subclass: x,
            superclass: target,
        })
        .unwrap();
    }

    let full_after = classify_full(reasoner.ontology());
    let incr_after = classify_incremental(&mut reasoner);
    assert_taxonomy_eq(&full_after, &incr_after);
}

#[test]
fn incremental_matches_full_after_axiom_removal() {
    let mut ontology = Ontology::new();
    let a = class(&mut ontology, "http://ex.org/A");
    let b = class(&mut ontology, "http://ex.org/B");
    let c = class(&mut ontology, "http://ex.org/C");
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
        .profile(Profile::El)
        .config(ReasonerConfig {
            incremental: true,
            ..ReasonerConfig::default()
        })
        .build(ontology)
        .unwrap();

    classify_incremental(&mut reasoner);
    reasoner.ontology_mut().remove_axiom(bc_id).unwrap();

    let full_after = classify_full(reasoner.ontology());
    let incr_after = classify_incremental(&mut reasoner);
    assert_taxonomy_eq(&full_after, &incr_after);
    assert!(!incr_after.is_subsumed(a, c));
}
