use ontologos_core::{EntityKind, Ontology};

const PIZZA: &str = "http://www.co-ode.org/ontologies/pizza/pizza.owl#Pizza";
const THING: &str = "http://www.w3.org/2002/07/owl#Thing";
const HAS_TOPPING: &str = "http://www.co-ode.org/ontologies/pizza/pizza.owl#hasTopping";
const PIZZA_TOPPING: &str = "http://www.co-ode.org/ontologies/pizza/pizza.owl#PizzaTopping";

#[test]
fn loads_pizza_minimal_fixture() {
    let json = include_str!("fixtures/pizza_minimal.json");
    let ontology = Ontology::from_json(json).expect("load fixture");

    assert_eq!(ontology.iri_count(), 4);
    assert_eq!(ontology.entity_count(), 4);
    assert_eq!(ontology.axiom_count(), 2);

    let pizza = ontology.lookup_entity(PIZZA).expect("pizza entity");
    let thing = ontology.lookup_entity(THING).expect("thing entity");
    assert_eq!(ontology.direct_superclasses(pizza), &[thing]);

    let has_topping = ontology.lookup_entity(HAS_TOPPING).expect("property");
    let topping_class = ontology
        .lookup_entity(PIZZA_TOPPING)
        .expect("topping class");
    assert_eq!(ontology.index().ranges_of(has_topping), &[topping_class]);
}

#[test]
fn builder_registers_pizza_entities() {
    let ontology = Ontology::builder()
        .class(PIZZA)
        .expect("class")
        .class(THING)
        .expect("class")
        .object_property(HAS_TOPPING)
        .expect("property")
        .class(PIZZA_TOPPING)
        .expect("class")
        .subclass_of(PIZZA, THING)
        .expect("subclass")
        .build()
        .expect("build");

    assert_eq!(ontology.entity_count(), 4);
    assert_eq!(ontology.axiom_count(), 1);

    let pizza = ontology.lookup_entity(PIZZA).expect("pizza");
    let record = ontology.entity(pizza).expect("record");
    assert_eq!(record.kind, EntityKind::Class);
}

#[test]
fn builder_json_matches_fixture_shape() {
    let mut ontology = Ontology::builder()
        .class(PIZZA)
        .expect("class")
        .class(THING)
        .expect("class")
        .object_property(HAS_TOPPING)
        .expect("property")
        .class(PIZZA_TOPPING)
        .expect("class")
        .subclass_of(PIZZA, THING)
        .expect("subclass")
        .build()
        .expect("build");

    let has_topping = ontology.lookup_entity(HAS_TOPPING).expect("property");
    let topping_class = ontology.lookup_entity(PIZZA_TOPPING).expect("topping");
    ontology
        .add_axiom(ontologos_core::Axiom::ObjectPropertyRange {
            property: has_topping,
            range: topping_class,
        })
        .expect("range");

    let from_fixture =
        Ontology::from_json(include_str!("fixtures/pizza_minimal.json")).expect("fixture");
    let from_builder =
        Ontology::from_json(&ontology.to_json().expect("json")).expect("builder json");

    assert_eq!(from_builder, from_fixture);
}
