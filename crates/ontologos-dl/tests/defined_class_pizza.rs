//! Pizza defined-class taxonomy enrichment smoke test.

use ontologos_dl::classify;
use ontologos_parser::load_ontology;

#[test]
fn pizza_defined_class_enrichment_adds_cheesey_subsumptions() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data/pizza.owl");
    assert!(
        path.exists(),
        "missing pizza.owl at {} (run ./benchmarks/scripts/download.sh)",
        path.display()
    );
    let ontology = load_ontology(&path).expect("load pizza");
    let ns = "https://raw.githubusercontent.com/owlcs/pizza-ontology/refs/heads/master/pizza.owl#";
    let american = ontology.lookup_entity(&format!("{ns}American")).unwrap();
    let cheesey = ontology
        .lookup_entity(&format!("{ns}CheeseyPizza"))
        .unwrap();
    let mozzarella = ontology
        .lookup_entity(&format!("{ns}MozzarellaTopping"))
        .unwrap();
    let cheese = ontology
        .lookup_entity(&format!("{ns}CheeseTopping"))
        .unwrap();
    let veg = ontology
        .lookup_entity(&format!("{ns}VegetarianTopping"))
        .unwrap();

    let taxonomy = classify(&ontology).expect("classify");
    eprintln!("count={}", taxonomy.subsumption_count());
    eprintln!(
        "american->cheesey={}",
        taxonomy.is_subsumed(american, cheesey)
    );
    eprintln!(
        "mozzarella->cheese={}",
        taxonomy.is_subsumed(mozzarella, cheese)
    );
    eprintln!("cheese->veg={}", taxonomy.is_subsumed(cheese, veg));
    assert!(
        taxonomy.is_subsumed(american, cheesey),
        "American should be subsumed by CheeseyPizza"
    );
}
