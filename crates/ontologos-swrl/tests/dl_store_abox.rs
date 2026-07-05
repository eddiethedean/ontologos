use ontologos_core::{ClassExpr, DlAxiom, EntityKind, Ontology, SwrlAtom, SwrlIArg, SwrlRule};
use ontologos_swrl::materialize_swrl_rules;

#[test]
fn forward_chain_matches_dl_store_class_assertion() {
    let mut ontology = Ontology::new();
    let person = ontology
        .entity_id("http://ex.org/Person", EntityKind::Class)
        .unwrap();
    let mortal = ontology
        .entity_id("http://ex.org/Mortal", EntityKind::Class)
        .unwrap();
    let alice = ontology
        .entity_id("http://ex.org/alice", EntityKind::Individual)
        .unwrap();
    let ce = ontology.dl_mut().intern_ce(ClassExpr::Atomic(person));
    ontology.dl_mut().push_axiom(DlAxiom::ClassAssertion {
        individual: alice,
        class: ce,
    });
    ontology.reindex_dl_abox();
    ontology
        .push_swrl_rule(SwrlRule {
            body: vec![SwrlAtom::Class {
                class: person,
                arg: SwrlIArg::Variable("x".into()),
            }],
            head: vec![SwrlAtom::Class {
                class: mortal,
                arg: SwrlIArg::Variable("x".into()),
            }],
        })
        .unwrap();

    let report = materialize_swrl_rules(&mut ontology).expect("materialize");
    assert!(
        report.inferences_added > 0,
        "expected Person(x) match from DL-store ClassAssertion"
    );
}
