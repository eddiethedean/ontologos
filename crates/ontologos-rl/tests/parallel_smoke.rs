use ontologos_core::{Axiom, EntityKind, Ontology};
use ontologos_rl::RlEngine;

fn build_fixture(n: usize) -> Ontology {
    let mut ontology = Ontology::new();
    let person = ontology
        .entity_id("http://example.org/Person", EntityKind::Class)
        .expect("Person");
    let knows = ontology
        .entity_id("http://example.org/knows", EntityKind::ObjectProperty)
        .expect("knows");
    ontology
        .add_axiom(Axiom::ObjectPropertyDomain {
            property: knows,
            domain: person,
        })
        .expect("domain");
    ontology
        .add_axiom(Axiom::ObjectPropertyRange {
            property: knows,
            range: person,
        })
        .expect("range");

    for i in 0..n {
        let a = ontology
            .entity_id(&format!("http://example.org/i{i}"), EntityKind::Individual)
            .expect("a");
        let b = ontology
            .entity_id(
                &format!("http://example.org/i{}", (i + 1) % n),
                EntityKind::Individual,
            )
            .expect("b");
        ontology
            .add_axiom(Axiom::ObjectPropertyAssertion {
                subject: a,
                property: knows,
                object: b,
            })
            .expect("assertion");
    }
    ontology
}

#[test]
#[ignore = "reasonable adapter ignores parallelism param; both paths use the same engine"]
fn parallel_produces_same_saturation_as_sequential() {
    let base = build_fixture(2_000);
    let mut seq = base.clone();
    let mut par = base;
    RlEngine::new(1).saturate(&mut seq).expect("seq");
    RlEngine::try_new(4)
        .expect("engine")
        .saturate(&mut par)
        .expect("par");
    assert_eq!(seq, par);
}
