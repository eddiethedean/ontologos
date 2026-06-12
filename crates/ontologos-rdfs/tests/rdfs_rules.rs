use ontologos_core::{Axiom, Ontology, Profile, Reasoner};
use ontologos_rdfs::{materialize_reasoner, MaterializationReport, RdfsEngine, RdfsRule};

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
        report
            .inferred_by_rule
            .get(&RdfsRule::ScTrans)
            .copied()
            .unwrap_or(0)
            >= 1
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
        report
            .inferred_by_rule
            .get(&RdfsRule::SpTrans)
            .copied()
            .unwrap_or(0)
            >= 1
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
        report
            .inferred_by_rule
            .get(&RdfsRule::DomInherit)
            .copied()
            .unwrap_or(0)
            >= 1
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
        report
            .inferred_by_rule
            .get(&RdfsRule::RngInherit)
            .copied()
            .unwrap_or(0)
            >= 1
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
