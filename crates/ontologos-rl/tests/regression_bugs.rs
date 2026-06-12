//! Regression tests for confirmed RL bugs tracked on GitHub.
//! Run ignored tests: `cargo test -p ontologos-rl --test regression_bugs -- --ignored`

use ontologos_core::{Axiom, EntityId, Ontology};
use ontologos_rl::RlEngine;

const NS: &str = "http://example.org/regression#";

fn iri(local: &str) -> String {
    format!("{NS}{local}")
}

fn is_typed(ontology: &Ontology, individual: &str, class: &str) -> bool {
    let Some(ind) = ontology.lookup_entity(individual) else {
        return false;
    };
    let Some(class_id) = ontology.lookup_entity(class) else {
        return false;
    };
    fn subsumed(ontology: &Ontology, subclass: EntityId, superclass: EntityId) -> bool {
        if subclass == superclass {
            return true;
        }
        ontology
            .direct_superclasses(subclass)
            .iter()
            .any(|&sup| subsumed(ontology, sup, superclass))
    }
    ontology
        .classes_of(ind)
        .iter()
        .any(|&c| c == class_id || subsumed(ontology, c, class_id))
}

fn saturate(ontology: &mut Ontology) {
    RlEngine::new(1).saturate(ontology).expect("saturate");
}

/// Domain on subproperty Q should type assertion on superproperty P (prp-dom2).
/// RDFS dom_inherit runs once before RL; no domain is copied to P when only Q has domain.
#[test]
#[ignore = "bug #1: RL TypeDomain misses domain when subproperty has domain"]
fn domain_on_subproperty_types_superproperty_assertion() {
    let mut ontology = Ontology::builder()
        .class(&iri("Person"))
        .expect("Person")
        .individual(&iri("a"))
        .expect("a")
        .individual(&iri("b"))
        .expect("b")
        .object_property(&iri("P"))
        .expect("P")
        .object_property(&iri("Q"))
        .expect("Q")
        .subproperty_of(&iri("Q"), &iri("P"))
        .expect("Q sub P")
        .property_domain(&iri("Q"), &iri("Person"))
        .expect("domain on Q")
        .object_property_assertion(&iri("a"), &iri("P"), &iri("b"))
        .expect("assertion on P")
        .build()
        .expect("build");

    saturate(&mut ontology);

    assert!(
        is_typed(&ontology, &iri("a"), &iri("Person")),
        "expected TypeDomain: domain on subproperty Q should apply to assertion on superproperty P"
    );
}

/// Documents that direct `disjoint_with(B)` does not see `D` when only `A` is disjoint with `D`.
/// Full saturation may still report a clash via `TypeSubclass` + `EqClassSub` (see issue #5).
#[test]
fn disjoint_index_does_not_expand_equivalent_classes() {
    let mut ontology = Ontology::builder()
        .class(&iri("A"))
        .expect("A")
        .class(&iri("B"))
        .expect("B")
        .class(&iri("D"))
        .expect("D")
        .build()
        .expect("build");

    let a = ontology.lookup_entity(&iri("A")).expect("A");
    let b = ontology.lookup_entity(&iri("B")).expect("B");
    let d = ontology.lookup_entity(&iri("D")).expect("D");
    ontology
        .add_axiom(Axiom::EquivalentClasses(vec![a, b]))
        .expect("equiv");
    ontology
        .add_axiom(Axiom::DisjointClasses(vec![a, d]))
        .expect("disjoint");

    let direct_disjoint = ontology
        .disjoint_with(b)
        .is_some_and(|set| set.contains(&d));
    assert!(
        !direct_disjoint,
        "setup: index disjoint_with(B) should not list D when only A disjoint D"
    );
}
