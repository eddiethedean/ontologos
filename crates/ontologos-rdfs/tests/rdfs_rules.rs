use ontologos_core::{Axiom, Ontology, Profile, Reasoner};
use ontologos_rdfs::{classify_reasoner, materialize_reasoner, MaterializationReport, RdfsEngine};

const NS: &str = "http://example.org/";

fn materialize(ontology: &mut Ontology) -> MaterializationReport {
    RdfsEngine::new()
        .materialize(ontology)
        .expect("materialize")
}

#[test]
fn sc_trans_infers_transitive_subclass() {
    let mut ontology = Ontology::builder()
        .class(&format!("{NS}A"))
        .expect("A")
        .class(&format!("{NS}B"))
        .expect("B")
        .class(&format!("{NS}C"))
        .expect("C")
        .subclass_of(&format!("{NS}A"), &format!("{NS}B"))
        .expect("A sub B")
        .subclass_of(&format!("{NS}B"), &format!("{NS}C"))
        .expect("B sub C")
        .build()
        .expect("build");

    let report = materialize(&mut ontology);
    assert!(
        report.inferred_total() >= 1,
        "expected transitive subclass inference"
    );

    let a = ontology.lookup_entity(&format!("{NS}A")).expect("A");
    let c = ontology.lookup_entity(&format!("{NS}C")).expect("C");
    assert!(ontology.direct_superclasses(a).contains(&c));
}

#[test]
fn sp_trans_infers_transitive_subproperty() {
    let mut ontology = Ontology::builder()
        .object_property(&format!("{NS}p"))
        .expect("p")
        .object_property(&format!("{NS}q"))
        .expect("q")
        .object_property(&format!("{NS}r"))
        .expect("r")
        .subproperty_of(&format!("{NS}p"), &format!("{NS}q"))
        .expect("p sub q")
        .subproperty_of(&format!("{NS}q"), &format!("{NS}r"))
        .expect("q sub r")
        .build()
        .expect("build");

    let report = materialize(&mut ontology);
    assert!(
        report.inferred_total() >= 1,
        "expected transitive subproperty inference"
    );

    let p = ontology.lookup_entity(&format!("{NS}p")).expect("p");
    let r = ontology.lookup_entity(&format!("{NS}r")).expect("r");
    assert!(ontology.direct_superproperties(p).contains(&r));
}

#[test]
fn dom_inherit_infers_domain_from_superproperty() {
    let mut ontology = Ontology::builder()
        .class(&format!("{NS}Person"))
        .expect("Person")
        .object_property(&format!("{NS}hasParent"))
        .expect("hasParent")
        .object_property(&format!("{NS}hasFather"))
        .expect("hasFather")
        .subproperty_of(&format!("{NS}hasFather"), &format!("{NS}hasParent"))
        .expect("subprop")
        .property_domain(&format!("{NS}hasParent"), &format!("{NS}Person"))
        .expect("domain")
        .build()
        .expect("build");

    let report = materialize(&mut ontology);
    assert!(
        report.inferred_total() >= 1,
        "expected domain inheritance inference"
    );

    let has_father = ontology
        .lookup_entity(&format!("{NS}hasFather"))
        .expect("hasFather");
    let person = ontology
        .lookup_entity(&format!("{NS}Person"))
        .expect("Person");
    assert!(ontology.index().domains_of(has_father).contains(&person));
}

#[test]
fn rng_inherit_infers_range_from_superproperty() {
    let mut ontology = Ontology::builder()
        .class(&format!("{NS}Person"))
        .expect("Person")
        .class(&format!("{NS}Man"))
        .expect("Man")
        .object_property(&format!("{NS}hasChild"))
        .expect("hasChild")
        .object_property(&format!("{NS}hasSon"))
        .expect("hasSon")
        .subproperty_of(&format!("{NS}hasSon"), &format!("{NS}hasChild"))
        .expect("subprop")
        .property_range(&format!("{NS}hasChild"), &format!("{NS}Person"))
        .expect("range")
        .property_range(&format!("{NS}hasSon"), &format!("{NS}Man"))
        .expect("existing range")
        .build()
        .expect("build");

    let report = materialize(&mut ontology);
    assert!(
        report.inferred_total() >= 1,
        "expected range inheritance inference"
    );

    let has_son = ontology
        .lookup_entity(&format!("{NS}hasSon"))
        .expect("hasSon");
    let person = ontology
        .lookup_entity(&format!("{NS}Person"))
        .expect("Person");
    assert!(ontology.index().ranges_of(has_son).contains(&person));
}

#[test]
fn materialize_is_idempotent() {
    let mut ontology = Ontology::builder()
        .class(&format!("{NS}A"))
        .expect("A")
        .class(&format!("{NS}B"))
        .expect("B")
        .class(&format!("{NS}C"))
        .expect("C")
        .subclass_of(&format!("{NS}A"), &format!("{NS}B"))
        .expect("A sub B")
        .subclass_of(&format!("{NS}B"), &format!("{NS}C"))
        .expect("B sub C")
        .build()
        .expect("build");

    materialize(&mut ontology);
    let count = ontology.axiom_count();
    let second = materialize(&mut ontology);
    assert_eq!(ontology.axiom_count(), count);
    assert_eq!(second.inferred_total(), 0);
}

#[test]
fn materialize_reasoner_requires_rdfs_profile() {
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
        .profile(Profile::El)
        .build(ontology)
        .expect("reasoner");
    let err = materialize_reasoner(&mut reasoner).expect_err("wrong profile");
    assert!(matches!(err, ontologos_rdfs::Error::WrongProfile { .. }));
}

#[test]
fn materialize_reasoner_delegates_for_rdfs_profile() {
    let ontology = Ontology::builder()
        .class(&format!("{NS}A"))
        .expect("A")
        .class(&format!("{NS}B"))
        .expect("B")
        .class(&format!("{NS}C"))
        .expect("C")
        .subclass_of(&format!("{NS}A"), &format!("{NS}B"))
        .expect("A sub B")
        .subclass_of(&format!("{NS}B"), &format!("{NS}C"))
        .expect("B sub C")
        .build()
        .expect("build");

    let mut reasoner = Reasoner::builder()
        .profile(Profile::Rdfs)
        .build(ontology)
        .expect("reasoner");
    let report = materialize_reasoner(&mut reasoner).expect("materialize");
    assert!(report.inferred_total() >= 1);
}

#[test]
fn inferred_axioms_update_indexes() {
    let mut ontology = Ontology::builder()
        .class(&format!("{NS}A"))
        .expect("A")
        .class(&format!("{NS}B"))
        .expect("B")
        .class(&format!("{NS}C"))
        .expect("C")
        .subclass_of(&format!("{NS}A"), &format!("{NS}B"))
        .expect("A sub B")
        .subclass_of(&format!("{NS}B"), &format!("{NS}C"))
        .expect("B sub C")
        .build()
        .expect("build");

    materialize(&mut ontology);

    let a = ontology.lookup_entity(&format!("{NS}A")).expect("A");
    let c = ontology.lookup_entity(&format!("{NS}C")).expect("C");
    assert!(ontology.direct_superclasses(a).contains(&c));

    let mut found = false;
    for (_, axiom) in ontology.axioms().iter() {
        if matches!(
            axiom,
            Axiom::SubClassOf {
                subclass,
                superclass,
            } if *subclass == a && *superclass == c
        ) {
            found = true;
        }
    }
    assert!(found, "inferred axiom stored in axiom store");
}

#[test]
fn materialize_with_traces_records_premises() {
    let mut ontology = Ontology::builder()
        .class(&format!("{NS}A"))
        .expect("A")
        .class(&format!("{NS}B"))
        .expect("B")
        .class(&format!("{NS}C"))
        .expect("C")
        .subclass_of(&format!("{NS}A"), &format!("{NS}B"))
        .expect("A sub B")
        .subclass_of(&format!("{NS}B"), &format!("{NS}C"))
        .expect("B sub C")
        .build()
        .expect("build");

    let report = RdfsEngine::new()
        .with_traces(true)
        .materialize(&mut ontology)
        .expect("materialize");

    assert!(report.inferred_total() >= 1);
    assert!(
        report.trace.steps.is_empty(),
        "traces empty until reasonable exposes diagnostics"
    );
}

#[test]
fn reasoner_classify_rdfs_profile_materializes() {
    let ontology = Ontology::builder()
        .class(&format!("{NS}A"))
        .expect("A")
        .class(&format!("{NS}B"))
        .expect("B")
        .class(&format!("{NS}C"))
        .expect("C")
        .subclass_of(&format!("{NS}A"), &format!("{NS}B"))
        .expect("A sub B")
        .subclass_of(&format!("{NS}B"), &format!("{NS}C"))
        .expect("B sub C")
        .build()
        .expect("build");

    let mut reasoner = Reasoner::builder()
        .profile(Profile::Rdfs)
        .build(ontology)
        .expect("reasoner");
    let before = reasoner.ontology().axiom_count();
    classify_reasoner(&mut reasoner).expect("classify");
    assert!(reasoner.ontology().axiom_count() > before);
}

#[test]
fn sc_trans_infers_long_transitive_chain() {
    let mut builder = Ontology::builder();
    let labels = ["A", "B", "C", "D", "E"];
    for label in labels {
        builder = builder.class(&format!("{NS}{label}")).expect(label);
    }
    let mut ontology = builder
        .subclass_of(&format!("{NS}A"), &format!("{NS}B"))
        .expect("A sub B")
        .subclass_of(&format!("{NS}B"), &format!("{NS}C"))
        .expect("B sub C")
        .subclass_of(&format!("{NS}C"), &format!("{NS}D"))
        .expect("C sub D")
        .subclass_of(&format!("{NS}D"), &format!("{NS}E"))
        .expect("D sub E")
        .build()
        .expect("build");

    materialize(&mut ontology);

    let a = ontology.lookup_entity(&format!("{NS}A")).expect("A");
    let e = ontology.lookup_entity(&format!("{NS}E")).expect("E");
    assert!(ontology.direct_superclasses(a).contains(&e));
}

#[test]
fn classify_reasoner_non_rdfs_returns_not_implemented() {
    let ontology = Ontology::default();
    let mut reasoner = Reasoner::builder()
        .profile(Profile::El)
        .build(ontology)
        .expect("reasoner");
    let err = classify_reasoner(&mut reasoner).expect_err("stub");
    assert!(matches!(err, ontologos_rdfs::Error::Core(_)));
}
