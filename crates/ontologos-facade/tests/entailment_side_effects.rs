use ontologos_core::{Axiom, EntityKind, Ontology, Profile, Reasoner};
use ontologos_facade::{
    EntailmentCheck, get_object_property_values, is_entailed_axiom, is_subsumption_entailed,
};

#[test]
fn subsumption_entailment_rl_does_not_mutate_shared_ontology() {
    let ontology = Ontology::builder()
        .class("http://ex.org/A")
        .unwrap()
        .class("http://ex.org/B")
        .unwrap()
        .class("http://ex.org/C")
        .unwrap()
        .subclass_of("http://ex.org/A", "http://ex.org/B")
        .unwrap()
        .subclass_of("http://ex.org/B", "http://ex.org/C")
        .unwrap()
        .build()
        .unwrap();
    let before = ontology.axiom_count();
    let mut reasoner = Reasoner::builder()
        .profile(Profile::Rl)
        .build(ontology)
        .unwrap();
    assert!(is_subsumption_entailed(&mut reasoner, "http://ex.org/A", "http://ex.org/C").unwrap());
    assert_eq!(reasoner.ontology().axiom_count(), before);
}

#[test]
fn class_assertion_entailment_rdfs_materializes_subclass_typing() {
    let mut ontology = Ontology::new();
    let a = ontology
        .entity_id("http://ex.org/A", EntityKind::Class)
        .unwrap();
    let b = ontology
        .entity_id("http://ex.org/B", EntityKind::Class)
        .unwrap();
    let x = ontology
        .entity_id("http://ex.org/x", EntityKind::Individual)
        .unwrap();
    ontology
        .add_axiom(Axiom::SubClassOf {
            subclass: a,
            superclass: b,
        })
        .unwrap();
    ontology
        .add_axiom(Axiom::ClassAssertion {
            individual: x,
            class: a,
        })
        .unwrap();
    let before = ontology.axiom_count();
    let mut reasoner = Reasoner::builder()
        .profile(Profile::Rdfs)
        .build(ontology)
        .unwrap();
    assert!(
        is_entailed_axiom(
            &mut reasoner,
            EntailmentCheck::ClassAssertion {
                individual: "http://ex.org/x".into(),
                class: "http://ex.org/B".into(),
            }
        )
        .unwrap()
    );
    assert_eq!(reasoner.ontology().axiom_count(), before);
}

#[test]
fn class_assertion_entailment_follows_same_as() {
    let mut ontology = Ontology::new();
    let a = ontology
        .entity_id("http://ex.org/A", EntityKind::Class)
        .unwrap();
    let x = ontology
        .entity_id("http://ex.org/x", EntityKind::Individual)
        .unwrap();
    let y = ontology
        .entity_id("http://ex.org/y", EntityKind::Individual)
        .unwrap();
    ontology
        .add_axiom(Axiom::SameIndividual(vec![x, y]))
        .unwrap();
    ontology
        .add_axiom(Axiom::ClassAssertion {
            individual: x,
            class: a,
        })
        .unwrap();
    let mut reasoner = Reasoner::builder()
        .profile(Profile::El)
        .build(ontology)
        .unwrap();
    assert!(
        is_entailed_axiom(
            &mut reasoner,
            EntailmentCheck::ClassAssertion {
                individual: "http://ex.org/y".into(),
                class: "http://ex.org/A".into(),
            }
        )
        .unwrap()
    );
}

#[test]
fn object_property_lookup_does_not_mutate_shared_ontology() {
    let mut ontology = Ontology::new();
    let c = ontology
        .entity_id("http://ex.org/c", EntityKind::Individual)
        .unwrap();
    let d = ontology
        .entity_id("http://ex.org/d", EntityKind::Individual)
        .unwrap();
    let p = ontology
        .entity_id("http://ex.org/p", EntityKind::ObjectProperty)
        .unwrap();
    ontology
        .add_axiom(Axiom::ObjectPropertyAssertion {
            subject: c,
            property: p,
            object: d,
        })
        .unwrap();
    let before = ontology.axiom_count();
    let reasoner = Reasoner::builder()
        .profile(Profile::Rdfs)
        .build(ontology)
        .unwrap();
    let _ = get_object_property_values(&reasoner, "http://ex.org/c", "http://ex.org/p").unwrap();
    assert_eq!(reasoner.ontology().axiom_count(), before);
}

#[test]
fn rl_object_property_lookup_applies_symmetric_expansion() {
    let mut ontology = Ontology::new();
    let c = ontology
        .entity_id("http://ex.org/c", EntityKind::Individual)
        .unwrap();
    let d = ontology
        .entity_id("http://ex.org/d", EntityKind::Individual)
        .unwrap();
    let p = ontology
        .entity_id("http://ex.org/p", EntityKind::ObjectProperty)
        .unwrap();
    ontology
        .add_axiom(Axiom::SymmetricObjectProperty(p))
        .unwrap();
    ontology
        .add_axiom(Axiom::ObjectPropertyAssertion {
            subject: c,
            property: p,
            object: d,
        })
        .unwrap();
    let reasoner = Reasoner::builder()
        .profile(Profile::Rl)
        .build(ontology)
        .unwrap();
    let values =
        get_object_property_values(&reasoner, "http://ex.org/d", "http://ex.org/p").unwrap();
    assert_eq!(values, vec!["http://ex.org/c"]);
}
