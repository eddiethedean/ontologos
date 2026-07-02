//! B-03: transitive sameAs closure in SWRL rule matching.

use ontologos_core::{Axiom, EntityKind, Ontology, SwrlAtom, SwrlIArg, SwrlRule};
use ontologos_swrl::materialize_swrl_rules;

#[test]
fn same_as_chain_matches_rule_head() {
    let mut ontology = Ontology::new();
    let x = ontology
        .entity_id("http://ex/x", EntityKind::Individual)
        .unwrap();
    let y = ontology
        .entity_id("http://ex/y", EntityKind::Individual)
        .unwrap();
    let z = ontology
        .entity_id("http://ex/z", EntityKind::Individual)
        .unwrap();
    let c = ontology
        .entity_id("http://ex/C", EntityKind::Class)
        .unwrap();
    ontology
        .add_axiom(Axiom::SameIndividual(vec![x, y]))
        .unwrap();
    ontology
        .add_axiom(Axiom::SameIndividual(vec![y, z]))
        .unwrap();
    ontology
        .add_axiom(Axiom::ClassAssertion {
            individual: x,
            class: c,
        })
        .unwrap();
    ontology
        .push_swrl_rule(SwrlRule {
            body: vec![SwrlAtom::Class {
                class: c,
                arg: SwrlIArg::Variable("a".into()),
            }],
            head: vec![SwrlAtom::Class {
                class: c,
                arg: SwrlIArg::Individual(z),
            }],
        })
        .unwrap();
    let report = materialize_swrl_rules(&mut ontology).expect("materialize");
    assert!(
        report.inferences_added >= 1,
        "rule should match x and z via sameAs chain"
    );
    assert!(ontology.axioms().iter().any(|(_, ax)| {
        matches!(
            ax,
            Axiom::ClassAssertion {
                individual,
                class: cls,
            } if *individual == z && *cls == c
        )
    }));
}
