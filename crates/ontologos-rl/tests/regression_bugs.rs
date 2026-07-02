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
#[test]
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

/// Domain on Q should type assertion on equivalent property P (EqPropSub in same round).
#[test]
fn equivalent_property_domain_types_sibling_property_assertion() {
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
        .equivalent_object_properties(&[&iri("P"), &iri("Q")])
        .expect("equiv")
        .property_domain(&iri("Q"), &iri("Person"))
        .expect("domain on Q")
        .object_property_assertion(&iri("a"), &iri("P"), &iri("b"))
        .expect("assertion on P")
        .build()
        .expect("build");

    saturate(&mut ontology);

    assert!(
        is_typed(&ontology, &iri("a"), &iri("Person")),
        "expected TypeDomain via equivalent property subproperty link"
    );
}

/// Domain on R should type assertion on P when R ⊑ Q ⊑ P (transitive subproperty chain).
#[test]
fn domain_on_transitive_subproperty_types_superproperty_assertion() {
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
        .object_property(&iri("R"))
        .expect("R")
        .subproperty_of(&iri("Q"), &iri("P"))
        .expect("Q sub P")
        .subproperty_of(&iri("R"), &iri("Q"))
        .expect("R sub Q")
        .property_domain(&iri("R"), &iri("Person"))
        .expect("domain on R")
        .object_property_assertion(&iri("a"), &iri("P"), &iri("b"))
        .expect("assertion on P")
        .build()
        .expect("build");

    saturate(&mut ontology);

    assert!(
        is_typed(&ontology, &iri("a"), &iri("Person")),
        "expected TypeDomain via transitive subproperty chain R ⊑ Q ⊑ P"
    );
}

/// Range on subproperty Q should type assertion object on superproperty P.
#[test]
fn range_on_subproperty_types_superproperty_assertion_object() {
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
        .property_range(&iri("Q"), &iri("Person"))
        .expect("range on Q")
        .object_property_assertion(&iri("a"), &iri("P"), &iri("b"))
        .expect("assertion on P")
        .build()
        .expect("build");

    saturate(&mut ontology);

    assert!(
        is_typed(&ontology, &iri("b"), &iri("Person")),
        "expected TypeRange: range on subproperty Q should apply to assertion on superproperty P"
    );
}

/// Direct `disjoint_with(B)` does not see `D` when only `A` is disjoint with `D`;
/// clash detection expands equivalence (issue #5 / cls-disjoint2).
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

/// End-to-end saturation check; the unit test `classes_are_disjoint_expands_equivalent_classes`
/// is the guard that catches reverts (TypeEquivalent may mask the gap in the same batch).
#[test]
fn disjoint_clash_detected_via_equivalent_class_expansion() {
    let mut ontology = Ontology::builder()
        .class(&iri("A"))
        .expect("A")
        .class(&iri("B"))
        .expect("B")
        .class(&iri("D"))
        .expect("D")
        .individual(&iri("x"))
        .expect("x")
        .class_assertion(&iri("x"), &iri("B"))
        .expect("x type B")
        .class_assertion(&iri("x"), &iri("D"))
        .expect("x type D")
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

    let report = RlEngine::new(1).saturate(&mut ontology).expect("saturate");

    assert!(
        !report.clashes.is_empty(),
        "expected disjoint clash when x typed B and D with A ≡ B and A ⊥ D"
    );
}

/// Equivalent types on one individual should produce one disjoint clash, not one per equivalent pair.
#[test]
fn disjoint_clash_deduped_for_equivalent_types_on_individual() {
    let mut ontology = Ontology::builder()
        .class(&iri("A"))
        .expect("A")
        .class(&iri("B"))
        .expect("B")
        .class(&iri("D"))
        .expect("D")
        .individual(&iri("x"))
        .expect("x")
        .class_assertion(&iri("x"), &iri("A"))
        .expect("x type A")
        .class_assertion(&iri("x"), &iri("B"))
        .expect("x type B")
        .class_assertion(&iri("x"), &iri("D"))
        .expect("x type D")
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

    let report = RlEngine::new(1).saturate(&mut ontology).expect("saturate");

    assert_eq!(
        report.clashes.len(),
        1,
        "expected one clash for A ≡ B typed with D, got: {:?}",
        report.clashes
    );
}

/// scm-spo1: existential on superclass propagates to subclass.
#[test]
fn existential_propagates_along_subclass_of() {
    let mut ontology = Ontology::builder()
        .class(&iri("Animal"))
        .expect("Animal")
        .class(&iri("Dog"))
        .expect("Dog")
        .class(&iri("Leg"))
        .expect("Leg")
        .object_property(&iri("hasLeg"))
        .expect("hasLeg")
        .subclass_of(&iri("Dog"), &iri("Animal"))
        .expect("Dog sub Animal")
        .build()
        .expect("build");

    let animal = ontology.lookup_entity(&iri("Animal")).expect("Animal");
    let has_leg = ontology.lookup_entity(&iri("hasLeg")).expect("hasLeg");
    let leg = ontology.lookup_entity(&iri("Leg")).expect("Leg");
    ontology
        .add_axiom(Axiom::SubClassOfExistential {
            subclass: animal,
            property: has_leg,
            filler: leg,
        })
        .expect("Animal exists hasLeg Leg");

    saturate(&mut ontology);

    let dog = ontology.lookup_entity(&iri("Dog")).expect("Dog");
    assert!(
        ontology.existentials_of(dog).contains(&(has_leg, leg)),
        "expected Dog ⊑ ∃hasLeg.Leg after scm-spo1"
    );
}

/// cls-svf2: filler subsumption enables existential subsumption between classes.
#[test]
fn existential_subsumption_with_filler_subclass() {
    let mut ontology = Ontology::builder()
        .class(&iri("A"))
        .expect("A")
        .class(&iri("B"))
        .expect("B")
        .class(&iri("D1"))
        .expect("D1")
        .class(&iri("D2"))
        .expect("D2")
        .object_property(&iri("R"))
        .expect("R")
        .subclass_of(&iri("D1"), &iri("D2"))
        .expect("D1 sub D2")
        .build()
        .expect("build");

    let a = ontology.lookup_entity(&iri("A")).expect("A");
    let b = ontology.lookup_entity(&iri("B")).expect("B");
    let d1 = ontology.lookup_entity(&iri("D1")).expect("D1");
    let d2 = ontology.lookup_entity(&iri("D2")).expect("D2");
    let r = ontology.lookup_entity(&iri("R")).expect("R");
    ontology
        .add_axiom(Axiom::SubClassOfExistential {
            subclass: a,
            property: r,
            filler: d1,
        })
        .expect("A exists R D1");
    ontology
        .add_axiom(Axiom::SubClassOfExistential {
            subclass: b,
            property: r,
            filler: d2,
        })
        .expect("B exists R D2");

    saturate(&mut ontology);

    assert!(
        ontology.direct_superclasses(a).contains(&b)
            || ontology.direct_superclasses(a).iter().any(|&sup| {
                let mut stack = vec![sup];
                while let Some(c) = stack.pop() {
                    if c == b {
                        return true;
                    }
                    stack.extend_from_slice(ontology.direct_superclasses(c));
                }
                false
            }),
        "expected A ⊑ B from cls-svf2 filler subsumption"
    );
}

/// sameAs/differentFrom clash should be reported once across saturation iterations.
#[test]
#[ignore = "reasonable does not surface sameAs/differentFrom clashes in MaterializationReport::clashes — see docs/reference/reasonable-limits.md"]
fn same_as_different_from_clash_deduped_across_iterations() {
    let mut ontology = Ontology::builder()
        .individual(&iri("a"))
        .expect("a")
        .individual(&iri("b"))
        .expect("b")
        .build()
        .expect("build");

    let a = ontology.lookup_entity(&iri("a")).expect("a");
    let b = ontology.lookup_entity(&iri("b")).expect("b");
    ontology
        .add_axiom(Axiom::SameIndividual(vec![a, b]))
        .expect("same");
    ontology
        .add_axiom(Axiom::DifferentIndividuals(vec![a, b]))
        .expect("different");

    let report = RlEngine::new(1).saturate(&mut ontology).expect("saturate");

    assert_eq!(
        report.clashes.len(),
        1,
        "expected one sameAs/differentFrom clash, got: {:?}",
        report.clashes
    );
}
