use ontologos_core::{Axiom, Ontology, Profile, Reasoner};
use ontologos_rl::{MaterializationReport, RlEngine, classify_reasoner, materialize_reasoner};

const NS: &str = "http://example.org/";

fn build_with_equivalent_classes() -> Ontology {
    let mut ontology = Ontology::builder()
        .class(&format!("{NS}A"))
        .expect("A")
        .class(&format!("{NS}B"))
        .expect("B")
        .build()
        .expect("build");

    let a = ontology.lookup_entity(&format!("{NS}A")).expect("A");
    let b = ontology.lookup_entity(&format!("{NS}B")).expect("B");
    ontology
        .add_axiom(Axiom::EquivalentClasses(vec![a, b]))
        .expect("equiv");
    ontology
}

fn saturate(ontology: &mut Ontology) -> MaterializationReport {
    RlEngine::new(1).saturate(ontology).expect("saturate")
}

#[test]
fn eq_class_sub_infers_mutual_subsumption() {
    let mut ontology = build_with_equivalent_classes();

    let report = saturate(&mut ontology);
    assert!(
        report.inferred_total() >= 1,
        "expected equivalent class propagation"
    );

    let a = ontology.lookup_entity(&format!("{NS}A")).expect("A");
    let b = ontology.lookup_entity(&format!("{NS}B")).expect("B");
    assert!(ontology.direct_superclasses(a).contains(&b));
    assert!(ontology.direct_superclasses(b).contains(&a));
}

#[test]
fn saturate_is_idempotent() {
    let mut ontology = build_with_equivalent_classes();

    saturate(&mut ontology);
    let count = ontology.axiom_count();
    let second = saturate(&mut ontology);
    assert_eq!(ontology.axiom_count(), count);
    assert_eq!(second.inferred_total(), 0);
}

#[test]
fn materialize_reasoner_requires_rl_profile() {
    let ontology = Ontology::builder()
        .class(&format!("{NS}A"))
        .expect("A")
        .class(&format!("{NS}B"))
        .expect("B")
        .subclass_of(&format!("{NS}A"), &format!("{NS}B"))
        .expect("sub")
        .build()
        .expect("build");

    let mut reasoner = Reasoner::builder()
        .profile(Profile::Rdfs)
        .build(ontology)
        .expect("reasoner");
    let err = materialize_reasoner(&mut reasoner).expect_err("wrong profile");
    assert!(matches!(err, ontologos_rl::Error::WrongProfile { .. }));
}

#[test]
fn materialize_reasoner_delegates_for_rl_profile() {
    let ontology = build_with_equivalent_classes();

    let mut reasoner = Reasoner::builder()
        .profile(Profile::Rl)
        .build(ontology)
        .expect("reasoner");
    let report = materialize_reasoner(&mut reasoner).expect("materialize");
    assert!(report.inferred_total() >= 1);
}

#[test]
fn classify_reasoner_non_rl_returns_not_implemented() {
    let ontology = Ontology::default();
    let mut reasoner = Reasoner::builder()
        .profile(Profile::El)
        .build(ontology)
        .expect("reasoner");
    let err = classify_reasoner(&mut reasoner).expect_err("stub");
    assert!(matches!(err, ontologos_rl::Error::Core(_)));
}

#[test]
fn inferred_axioms_update_indexes() {
    let mut ontology = build_with_equivalent_classes();

    saturate(&mut ontology);

    let a = ontology.lookup_entity(&format!("{NS}A")).expect("A");
    let b = ontology.lookup_entity(&format!("{NS}B")).expect("B");
    assert!(ontology.direct_superclasses(a).contains(&b));

    let mut found = false;
    for (_, axiom) in ontology.axioms().iter() {
        if matches!(
            axiom,
            Axiom::SubClassOf {
                subclass,
                superclass,
            } if *subclass == a && *superclass == b
        ) {
            found = true;
        }
    }
    assert!(found, "inferred axiom stored in axiom store");
}
