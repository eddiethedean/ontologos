//! Build a small pizza taxonomy and print a JSON round-trip summary.
//!
//! Run: `cargo run -p ontologos-core --example pizza_builder`

use ontologos_core::{Error, Ontology};

const PIZZA: &str = "http://www.co-ode.org/ontologies/pizza/pizza.owl#Pizza";
const THING: &str = "http://www.w3.org/2002/07/owl#Thing";
const HAS_TOPPING: &str = "http://www.co-ode.org/ontologies/pizza/pizza.owl#hasTopping";
const TOPPING: &str = "http://www.co-ode.org/ontologies/pizza/pizza.owl#PizzaTopping";

fn main() -> Result<(), Error> {
    let mut ontology = Ontology::builder()
        .class(PIZZA)?
        .class(THING)?
        .object_property(HAS_TOPPING)?
        .class(TOPPING)?
        .subclass_of(PIZZA, THING)?
        .build()?;

    let has_topping = ontology.lookup_entity(HAS_TOPPING).expect("hasTopping");
    let topping = ontology.lookup_entity(TOPPING).expect("topping");
    ontology.add_axiom(ontologos_core::Axiom::ObjectPropertyRange {
        property: has_topping,
        range: topping,
    })?;

    let pizza = ontology.lookup_entity(PIZZA).expect("pizza");
    let thing = ontology.lookup_entity(THING).expect("thing");
    let supers = ontology.direct_superclasses(pizza);

    let json = ontology.to_json()?;
    let restored = Ontology::from_json(&json)?;

    println!("Pizza direct superclasses: {supers:?}");
    println!("Expected superclass IRI: {THING}");
    println!("Superclass matches Thing: {}", supers == [thing]);
    println!("Entities: {}", ontology.entity_count());
    println!("Axioms: {}", ontology.axiom_count());
    println!("JSON bytes: {}", json.len());
    println!("Round-trip equal: {}", restored == ontology);

    Ok(())
}
