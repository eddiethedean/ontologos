use ontologos_core::{Axiom, EntityId, EntityKind, Ontology};

#[test]
fn add_axiom_updates_indexes() {
    let ontology = Ontology::builder()
        .class("http://ex.org/A")
        .expect("class")
        .class("http://ex.org/B")
        .expect("class")
        .subclass_of("http://ex.org/A", "http://ex.org/B")
        .expect("subclass")
        .build()
        .expect("build");

    let a = ontology.lookup_entity("http://ex.org/A").expect("A");
    let b = ontology.lookup_entity("http://ex.org/B").expect("B");
    assert_eq!(ontology.direct_superclasses(a), &[b]);
    assert_eq!(ontology.direct_subclasses(b), &[a]);
    assert_eq!(ontology.index().by_kind("SubClassOf").len(), 1);
}

#[test]
fn add_axiom_rejects_invalid_without_mutation() {
    let mut ontology = Ontology::builder()
        .class("http://ex.org/A")
        .expect("class")
        .build()
        .expect("build");

    let bad = Axiom::SubClassOf {
        subclass: EntityId(0),
        superclass: EntityId(99),
    };
    assert!(ontology.add_axiom(bad).is_err());
    assert_eq!(ontology.axiom_count(), 0);
    assert!(ontology.index().by_kind("SubClassOf").is_empty());
}

#[test]
fn add_axiom_subproperty_updates_index() {
    let mut ontology = Ontology::new();
    let sub = ontology
        .entity_id("http://ex.org/subProp", EntityKind::ObjectProperty)
        .expect("sub");
    let sup = ontology
        .entity_id("http://ex.org/superProp", EntityKind::ObjectProperty)
        .expect("super");
    ontology
        .add_axiom(Axiom::SubObjectPropertyOf {
            sub_property: sub,
            super_property: sup,
        })
        .expect("axiom");

    assert_eq!(ontology.index().direct_superproperties(sub), &[sup]);
}

#[test]
fn add_axiom_domain_updates_index() {
    let mut ontology = Ontology::new();
    let prop = ontology
        .entity_id("http://ex.org/prop", EntityKind::ObjectProperty)
        .expect("prop");
    let domain = ontology
        .entity_id("http://ex.org/Domain", EntityKind::Class)
        .expect("domain");
    ontology
        .add_axiom(Axiom::ObjectPropertyDomain {
            property: prop,
            domain,
        })
        .expect("axiom");

    assert_eq!(ontology.index().domains_of(prop), &[domain]);
}

#[test]
fn add_axiom_range_updates_index() {
    let mut ontology = Ontology::new();
    let prop = ontology
        .entity_id("http://ex.org/prop", EntityKind::ObjectProperty)
        .expect("prop");
    let range = ontology
        .entity_id("http://ex.org/Range", EntityKind::Class)
        .expect("range");
    ontology
        .add_axiom(Axiom::ObjectPropertyRange {
            property: prop,
            range,
        })
        .expect("axiom");

    assert_eq!(ontology.index().ranges_of(prop), &[range]);
}

#[test]
fn add_axiom_transitive_updates_index() {
    let mut ontology = Ontology::new();
    let prop = ontology
        .entity_id("http://ex.org/transProp", EntityKind::ObjectProperty)
        .expect("prop");
    ontology
        .add_axiom(Axiom::TransitiveObjectProperty(prop))
        .expect("axiom");

    assert!(ontology.index().transitive_properties().contains(&prop));
}
