use ontologos_core::{Axiom, EntityKind, Ontology};
use ontologos_el::ElClassifier;

#[test]
fn range_axiom_infers_filler_subsumption() {
    let mut ontology = Ontology::new();
    let c = ontology
        .entity_id("http://ex.org/C", EntityKind::Class)
        .expect("class");
    let d = ontology
        .entity_id("http://ex.org/D", EntityKind::Class)
        .expect("class");
    let r = ontology
        .entity_id("http://ex.org/r", EntityKind::ObjectProperty)
        .expect("property");
    ontology
        .add_axiom(Axiom::SubClassOfExistential {
            subclass: c,
            property: r,
            filler: c,
        })
        .expect("existential");
    ontology
        .add_axiom(Axiom::ObjectPropertyRange {
            property: r,
            range: d,
        })
        .expect("range");

    let taxonomy = ElClassifier::new().classify(&ontology).expect("classify");
    assert!(taxonomy.is_subsumed(c, d));
}
