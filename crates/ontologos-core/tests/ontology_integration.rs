use ontologos_core::{Axiom, EntityId, EntityKind, Error, Ontology};

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
    assert_eq!(ontology.direct_subproperties(sup), &[sub]);
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
fn add_axiom_equivalence_updates_index() {
    let mut ontology = Ontology::new();
    let a = ontology
        .entity_id("http://ex.org/A", EntityKind::Class)
        .expect("A");
    let b = ontology
        .entity_id("http://ex.org/B", EntityKind::Class)
        .expect("B");
    ontology
        .add_axiom(Axiom::EquivalentClasses(vec![a, b]))
        .expect("axiom");

    let equiv = ontology.equivalents_of(a).expect("equiv");
    assert!(equiv.contains(&b));
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

#[test]
fn add_axiom_subclass_of_existential_indexes_existentials() {
    let mut ontology = Ontology::new();
    let subclass = ontology
        .entity_id("http://ex.org/C", EntityKind::Class)
        .expect("subclass");
    let property = ontology
        .entity_id("http://ex.org/hasPart", EntityKind::ObjectProperty)
        .expect("property");
    let filler = ontology
        .entity_id("http://ex.org/B", EntityKind::Class)
        .expect("filler");
    ontology
        .add_axiom(Axiom::SubClassOfExistential {
            subclass,
            property,
            filler,
        })
        .expect("axiom");

    assert!(ontology.direct_superclasses(subclass).is_empty());
    assert!(ontology.index().ranges_of(property).is_empty());
    assert_eq!(ontology.existentials_of(subclass), &[(property, filler)]);
    assert_eq!(ontology.index().by_kind("SubClassOfExistential").len(), 1);
}

#[test]
fn add_axiom_symmetric_reflexive_functional_property_axioms() {
    let mut ontology = Ontology::new();
    let symmetric = ontology
        .entity_id("http://ex.org/symmetric", EntityKind::ObjectProperty)
        .expect("symmetric");
    let reflexive = ontology
        .entity_id("http://ex.org/reflexive", EntityKind::ObjectProperty)
        .expect("reflexive");
    let functional = ontology
        .entity_id("http://ex.org/functional", EntityKind::ObjectProperty)
        .expect("functional");

    ontology
        .add_axiom(Axiom::SymmetricObjectProperty(symmetric))
        .expect("symmetric");
    ontology
        .add_axiom(Axiom::ReflexiveObjectProperty(reflexive))
        .expect("reflexive");
    ontology
        .add_axiom(Axiom::FunctionalObjectProperty(functional))
        .expect("functional");

    assert_eq!(ontology.index().by_kind("SymmetricObjectProperty").len(), 1);
    assert_eq!(ontology.index().by_kind("ReflexiveObjectProperty").len(), 1);
    assert_eq!(
        ontology.index().by_kind("FunctionalObjectProperty").len(),
        1
    );
    assert!(ontology.index().symmetric_properties().contains(&symmetric));
    assert!(ontology.index().reflexive_properties().contains(&reflexive));
    assert!(
        ontology
            .index()
            .functional_properties()
            .contains(&functional)
    );
}

#[test]
fn add_axiom_rejects_conflicting_inverse_pairs() {
    let mut ontology = Ontology::new();
    let left = ontology
        .entity_id("http://ex.org/left", EntityKind::ObjectProperty)
        .expect("left");
    let right = ontology
        .entity_id("http://ex.org/right", EntityKind::ObjectProperty)
        .expect("right");
    let other = ontology
        .entity_id("http://ex.org/other", EntityKind::ObjectProperty)
        .expect("other");

    ontology
        .add_axiom(Axiom::InverseObjectProperties { left, right })
        .expect("first inverse");

    let err = ontology
        .add_axiom(Axiom::InverseObjectProperties { left, right: other })
        .expect_err("conflicting inverse");
    assert!(matches!(err, Error::InvalidAxiom(_)));
    assert_eq!(ontology.inverse_of(left), Some(right));
}
