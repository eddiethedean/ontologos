use ontologos_core::{EntityKind, Ontology};

#[test]
fn loads_pizza_minimal_fixture() {
    let json = include_str!("fixtures/pizza_minimal.json");
    let ontology = Ontology::from_json(json).expect("load fixture");

    assert_eq!(ontology.iri_count(), 4);
    assert_eq!(ontology.entity_count(), 4);
    assert_eq!(ontology.axiom_count(), 2);

    let pizza = ontology
        .lookup_entity("http://www.co-ode.org/ontologies/pizza/pizza.owl#Pizza")
        .expect("pizza entity");
    let thing = ontology
        .lookup_entity("http://www.w3.org/2002/07/owl#Thing")
        .expect("thing entity");
    assert_eq!(ontology.direct_superclasses(pizza), &[thing]);
}

#[test]
fn builder_matches_pizza_minimal_shape() {
    let ontology = Ontology::builder()
        .class("http://www.co-ode.org/ontologies/pizza/pizza.owl#Pizza")
        .expect("class")
        .class("http://www.w3.org/2002/07/owl#Thing")
        .expect("class")
        .object_property("http://www.co-ode.org/ontologies/pizza/pizza.owl#hasTopping")
        .expect("property")
        .class("http://www.co-ode.org/ontologies/pizza/pizza.owl#PizzaTopping")
        .expect("class")
        .subclass_of(
            "http://www.co-ode.org/ontologies/pizza/pizza.owl#Pizza",
            "http://www.w3.org/2002/07/owl#Thing",
        )
        .expect("subclass")
        .build()
        .expect("build");

    assert_eq!(ontology.entity_count(), 4);
    assert_eq!(ontology.axiom_count(), 1);

    let pizza = ontology
        .lookup_entity("http://www.co-ode.org/ontologies/pizza/pizza.owl#Pizza")
        .expect("pizza");
    let record = ontology.entity(pizza).expect("record");
    assert_eq!(record.kind, EntityKind::Class);
}
