//! Tier-A ports from HermiT `ReasonerTest` (OWL RL fragment).

use ontologos_conformance::{
    assert_object_property_assertion, assert_subproperty, assert_subsumed, assert_typed,
    has_property_characteristic, PropertyCharacteristic, PORT_NS,
};
use ontologos_core::{Axiom, Ontology};
use ontologos_rl::RlEngine;

fn iri(local: &str) -> String {
    format!("{PORT_NS}{local}")
}

fn saturate(ontology: &mut Ontology) {
    RlEngine::new(1).saturate(ontology).expect("saturate");
}

/// HermiT `ReasonerTest.testSubsumption2` (inlined existential encoding).
#[test]
#[ignore = "reasonable does not materialize named subClassOf from existential TBox patterns yet"]
fn subsumption2_property_subsumption_existential() {
    let mut ontology = Ontology::builder()
        .class(&iri("A"))
        .expect("A")
        .class(&iri("B"))
        .expect("B")
        .class(&iri("C"))
        .expect("C")
        .object_property(&iri("R"))
        .expect("R")
        .object_property(&iri("S"))
        .expect("S")
        .subproperty_of(&iri("R"), &iri("S"))
        .expect("R sub S")
        .build()
        .expect("build");

    let a = ontology.lookup_entity(&iri("A")).expect("A");
    let b = ontology.lookup_entity(&iri("B")).expect("B");
    let c = ontology.lookup_entity(&iri("C")).expect("C");
    let r = ontology.lookup_entity(&iri("R")).expect("R");
    let s = ontology.lookup_entity(&iri("S")).expect("S");

    ontology
        .add_axiom(ontologos_core::Axiom::SubClassOfExistential {
            subclass: a,
            property: r,
            filler: c,
        })
        .expect("exist A");
    ontology
        .add_axiom(ontologos_core::Axiom::SubClassOfExistential {
            subclass: b,
            property: s,
            filler: c,
        })
        .expect("exist B");

    saturate(&mut ontology);

    assert!(assert_subsumed(&ontology, &iri("A"), &iri("B")));
    assert!(!assert_subsumed(&ontology, &iri("B"), &iri("A")));
}

/// HermiT `ReasonerTest.testSubsumption3` (inlined equivalent properties).
#[test]
#[ignore = "reasonable does not materialize named subClassOf from existential TBox patterns yet"]
fn subsumption3_equivalent_properties_existential() {
    let mut ontology = Ontology::builder()
        .class(&iri("A"))
        .expect("A")
        .class(&iri("B"))
        .expect("B")
        .class(&iri("C"))
        .expect("C")
        .object_property(&iri("R"))
        .expect("R")
        .object_property(&iri("S"))
        .expect("S")
        .equivalent_object_properties(&[&iri("R"), &iri("S")])
        .expect("equiv props")
        .build()
        .expect("build");

    let a = ontology.lookup_entity(&iri("A")).expect("A");
    let b = ontology.lookup_entity(&iri("B")).expect("B");
    let c = ontology.lookup_entity(&iri("C")).expect("C");
    let r = ontology.lookup_entity(&iri("R")).expect("R");
    let s = ontology.lookup_entity(&iri("S")).expect("S");

    ontology
        .add_axiom(ontologos_core::Axiom::SubClassOfExistential {
            subclass: a,
            property: r,
            filler: c,
        })
        .expect("exist A");
    ontology
        .add_axiom(ontologos_core::Axiom::SubClassOfExistential {
            subclass: b,
            property: s,
            filler: c,
        })
        .expect("exist B");

    saturate(&mut ontology);

    assert!(assert_subsumed(&ontology, &iri("A"), &iri("B")));
    assert!(assert_subsumed(&ontology, &iri("B"), &iri("A")));
}

/// HermiT `ReasonerTest.testSameAs`
#[test]
fn same_as_propagates_class_assertion() {
    let mut ontology = Ontology::builder()
        .individual(&iri("a1"))
        .expect("a1")
        .individual(&iri("a2"))
        .expect("a2")
        .class(&iri("A"))
        .expect("A")
        .class_assertion(&iri("a1"), &iri("A"))
        .expect("type a1")
        .same_individual(&[&iri("a1"), &iri("a2")])
        .expect("same")
        .build()
        .expect("build");

    saturate(&mut ontology);
    assert!(assert_typed(&ontology, &iri("a2"), &iri("A")));
}

/// HermiT `ReasonerTest.testEquivalentClassInstances`
#[test]
fn equivalent_class_instances_share_types() {
    let mut ontology = Ontology::builder()
        .class(&iri("Car"))
        .expect("Car")
        .class(&iri("Automobile"))
        .expect("Automobile")
        .individual(&iri("car"))
        .expect("car")
        .individual(&iri("auto"))
        .expect("auto")
        .build()
        .expect("build");

    let car = ontology.lookup_entity(&iri("Car")).expect("Car");
    let automobile = ontology
        .lookup_entity(&iri("Automobile"))
        .expect("Automobile");
    ontology
        .add_axiom(Axiom::EquivalentClasses(vec![car, automobile]))
        .expect("equiv");
    ontology
        .add_axiom(Axiom::ClassAssertion {
            individual: ontology.lookup_entity(&iri("car")).expect("car"),
            class: car,
        })
        .expect("type car");
    ontology
        .add_axiom(Axiom::ClassAssertion {
            individual: ontology.lookup_entity(&iri("auto")).expect("auto"),
            class: automobile,
        })
        .expect("type auto");

    saturate(&mut ontology);

    assert!(assert_typed(&ontology, &iri("car"), &iri("Automobile")));
    assert!(assert_typed(&ontology, &iri("auto"), &iri("Car")));
}

/// HermiT `ReasonerTest.testReflexiveAndSameAs`
#[test]
fn reflexive_and_same_as_expand_property_instances() {
    let mut ontology = Ontology::builder()
        .individual(&iri("a"))
        .expect("a")
        .individual(&iri("b"))
        .expect("b")
        .object_property(&iri("r"))
        .expect("r")
        .object_property_assertion(&iri("a"), &iri("r"), &iri("b"))
        .expect("r a b")
        .same_individual(&[&iri("a"), &iri("b")])
        .expect("same")
        .build()
        .expect("build");

    let r = ontology.lookup_entity(&iri("r")).expect("r");
    ontology
        .add_axiom(Axiom::ReflexiveObjectProperty(r))
        .expect("reflexive");

    saturate(&mut ontology);

    assert!(assert_object_property_assertion(
        &ontology,
        &iri("a"),
        &iri("r"),
        &iri("b")
    ));
    assert!(assert_object_property_assertion(
        &ontology,
        &iri("b"),
        &iri("r"),
        &iri("a")
    ));
    assert!(assert_object_property_assertion(
        &ontology,
        &iri("a"),
        &iri("r"),
        &iri("a")
    ));
    assert!(assert_object_property_assertion(
        &ontology,
        &iri("b"),
        &iri("r"),
        &iri("b")
    ));
}

/// HermiT `ReasonerTest.testIndividualRetrievalBug`
#[test]
fn individual_property_retrieval() {
    let mut ontology = Ontology::builder()
        .class(&iri("A"))
        .expect("A")
        .individual(&iri("c"))
        .expect("c")
        .individual(&iri("d"))
        .expect("d")
        .object_property(&iri("r"))
        .expect("r")
        .class_assertion(&iri("d"), &iri("A"))
        .expect("type d")
        .object_property_assertion(&iri("c"), &iri("r"), &iri("d"))
        .expect("r c d")
        .build()
        .expect("build");

    saturate(&mut ontology);

    assert!(assert_object_property_assertion(
        &ontology,
        &iri("c"),
        &iri("r"),
        &iri("d")
    ));
}

/// HermiT `ReasonerTest.testIsFunctionalObject` (sub-property inherits functional).
#[test]
fn functional_property_characteristic_propagates_to_subproperty() {
    let mut ontology = Ontology::builder()
        .object_property(&iri("OP"))
        .expect("OP")
        .object_property(&iri("SOP"))
        .expect("SOP")
        .subproperty_of(&iri("SOP"), &iri("OP"))
        .expect("SOP sub OP")
        .build()
        .expect("build");

    let op = ontology.lookup_entity(&iri("OP")).expect("OP");
    ontology
        .add_axiom(Axiom::FunctionalObjectProperty(op))
        .expect("functional OP");

    saturate(&mut ontology);

    assert!(has_property_characteristic(
        &ontology,
        &iri("OP"),
        PropertyCharacteristic::Functional
    ));
    assert!(has_property_characteristic(
        &ontology,
        &iri("SOP"),
        PropertyCharacteristic::Functional
    ));
}

/// HermiT `ReasonerTest.testIsAsymmetricObject` (sub-property inherits asymmetric).
#[test]
fn asymmetric_property_characteristic_propagates_to_subproperty() {
    let mut ontology = Ontology::builder()
        .object_property(&iri("OP"))
        .expect("OP")
        .object_property(&iri("SOP1"))
        .expect("SOP1")
        .object_property(&iri("SOP2"))
        .expect("SOP2")
        .subproperty_of(&iri("SOP1"), &iri("OP"))
        .expect("SOP1 sub OP")
        .subproperty_of(&iri("OP"), &iri("SOP2"))
        .expect("OP sub SOP2")
        .build()
        .expect("build");

    let op = ontology.lookup_entity(&iri("OP")).expect("OP");
    ontology
        .add_axiom(Axiom::AsymmetricObjectProperty(op))
        .expect("asymmetric OP");

    saturate(&mut ontology);

    assert!(has_property_characteristic(
        &ontology,
        &iri("OP"),
        PropertyCharacteristic::Asymmetric
    ));
    assert!(has_property_characteristic(
        &ontology,
        &iri("SOP1"),
        PropertyCharacteristic::Asymmetric
    ));
    assert!(!has_property_characteristic(
        &ontology,
        &iri("SOP2"),
        PropertyCharacteristic::Asymmetric
    ));
}

/// RL: `subPropertyOf` propagates property assertions.
#[test]
fn property_assertion_propagates_along_subproperty() {
    let mut ontology = Ontology::builder()
        .individual(&iri("a"))
        .expect("a")
        .individual(&iri("b"))
        .expect("b")
        .object_property(&iri("r"))
        .expect("r")
        .object_property(&iri("s"))
        .expect("s")
        .subproperty_of(&iri("r"), &iri("s"))
        .expect("r sub s")
        .object_property_assertion(&iri("a"), &iri("r"), &iri("b"))
        .expect("r a b")
        .build()
        .expect("build");

    saturate(&mut ontology);

    assert!(assert_object_property_assertion(
        &ontology,
        &iri("a"),
        &iri("s"),
        &iri("b")
    ));
}

/// RL: inverse object properties swap assertion direction.
#[test]
fn inverse_property_assertion() {
    let mut ontology = Ontology::builder()
        .individual(&iri("a"))
        .expect("a")
        .individual(&iri("b"))
        .expect("b")
        .object_property(&iri("p"))
        .expect("p")
        .object_property(&iri("q"))
        .expect("q")
        .object_property_assertion(&iri("a"), &iri("p"), &iri("b"))
        .expect("p a b")
        .build()
        .expect("build");

    let p = ontology.lookup_entity(&iri("p")).expect("p");
    let q = ontology.lookup_entity(&iri("q")).expect("q");
    ontology
        .add_axiom(Axiom::InverseObjectProperties { left: p, right: q })
        .expect("inverse");

    saturate(&mut ontology);

    assert!(assert_object_property_assertion(
        &ontology,
        &iri("b"),
        &iri("q"),
        &iri("a")
    ));
}

/// RL: symmetric property yields reverse assertion.
#[test]
fn symmetric_property_assertion() {
    let mut ontology = Ontology::builder()
        .individual(&iri("a"))
        .expect("a")
        .individual(&iri("b"))
        .expect("b")
        .object_property(&iri("p"))
        .expect("p")
        .object_property_assertion(&iri("a"), &iri("p"), &iri("b"))
        .expect("p a b")
        .build()
        .expect("build");

    let p = ontology.lookup_entity(&iri("p")).expect("p");
    ontology
        .add_axiom(Axiom::SymmetricObjectProperty(p))
        .expect("symmetric");

    saturate(&mut ontology);

    assert!(assert_object_property_assertion(
        &ontology,
        &iri("b"),
        &iri("p"),
        &iri("a")
    ));
}

/// RL: transitive property chains assertions.
#[test]
fn transitive_property_assertion_chain() {
    let mut ontology = Ontology::builder()
        .individual(&iri("a"))
        .expect("a")
        .individual(&iri("b"))
        .expect("b")
        .individual(&iri("c"))
        .expect("c")
        .object_property(&iri("t"))
        .expect("t")
        .object_property_assertion(&iri("a"), &iri("t"), &iri("b"))
        .expect("t a b")
        .object_property_assertion(&iri("b"), &iri("t"), &iri("c"))
        .expect("t b c")
        .build()
        .expect("build");

    let t = ontology.lookup_entity(&iri("t")).expect("t");
    ontology
        .add_axiom(Axiom::TransitiveObjectProperty(t))
        .expect("transitive");

    saturate(&mut ontology);

    assert!(assert_object_property_assertion(
        &ontology,
        &iri("a"),
        &iri("t"),
        &iri("c")
    ));
}

/// RL: domain axiom types the subject of a property assertion.
#[test]
fn domain_types_property_assertion_subject() {
    let mut ontology = Ontology::builder()
        .class(&iri("Person"))
        .expect("Person")
        .individual(&iri("a"))
        .expect("a")
        .individual(&iri("b"))
        .expect("b")
        .object_property(&iri("knows"))
        .expect("knows")
        .property_domain(&iri("knows"), &iri("Person"))
        .expect("domain")
        .object_property_assertion(&iri("a"), &iri("knows"), &iri("b"))
        .expect("knows a b")
        .build()
        .expect("build");

    saturate(&mut ontology);

    assert!(assert_typed(&ontology, &iri("a"), &iri("Person")));
}

/// RL: domain on subproperty types subject of superproperty assertion (prp-dom2 + TypeDomain).
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

    assert!(assert_typed(&ontology, &iri("a"), &iri("Person")));
}

/// RL: range axiom types the object of a property assertion.
#[test]
fn range_types_property_assertion_object() {
    let mut ontology = Ontology::builder()
        .class(&iri("Person"))
        .expect("Person")
        .individual(&iri("a"))
        .expect("a")
        .individual(&iri("b"))
        .expect("b")
        .object_property(&iri("knows"))
        .expect("knows")
        .property_range(&iri("knows"), &iri("Person"))
        .expect("range")
        .object_property_assertion(&iri("a"), &iri("knows"), &iri("b"))
        .expect("knows a b")
        .build()
        .expect("build");

    saturate(&mut ontology);

    assert!(assert_typed(&ontology, &iri("b"), &iri("Person")));
}

/// RL: equivalent classes yield mutual `SubClassOf`.
#[test]
fn equivalent_classes_mutual_subclass() {
    let mut ontology = Ontology::builder()
        .class(&iri("A"))
        .expect("A")
        .class(&iri("B"))
        .expect("B")
        .build()
        .expect("build");

    let a = ontology.lookup_entity(&iri("A")).expect("A");
    let b = ontology.lookup_entity(&iri("B")).expect("B");
    ontology
        .add_axiom(Axiom::EquivalentClasses(vec![a, b]))
        .expect("equiv");

    saturate(&mut ontology);

    assert!(assert_subsumed(&ontology, &iri("A"), &iri("B")));
    assert!(assert_subsumed(&ontology, &iri("B"), &iri("A")));
}

/// RL: equivalent object properties yield mutual `SubObjectPropertyOf`.
#[test]
fn equivalent_properties_mutual_subproperty() {
    let mut ontology = Ontology::builder()
        .object_property(&iri("R"))
        .expect("R")
        .object_property(&iri("S"))
        .expect("S")
        .equivalent_object_properties(&[&iri("R"), &iri("S")])
        .expect("equiv")
        .build()
        .expect("build");

    saturate(&mut ontology);

    assert!(assert_subproperty(&ontology, &iri("R"), &iri("S")));
    assert!(assert_subproperty(&ontology, &iri("S"), &iri("R")));
}

/// RL: disjoint classes typed on one individual surface as a clash diagnostic.
#[test]
fn disjoint_classes_on_individual_report_clash() {
    let mut ontology = Ontology::builder()
        .class(&iri("A"))
        .expect("A")
        .class(&iri("B"))
        .expect("B")
        .individual(&iri("x"))
        .expect("x")
        .class_assertion(&iri("x"), &iri("A"))
        .expect("x type A")
        .class_assertion(&iri("x"), &iri("B"))
        .expect("x type B")
        .build()
        .expect("build");

    let a = ontology.lookup_entity(&iri("A")).expect("A");
    let b = ontology.lookup_entity(&iri("B")).expect("B");
    ontology
        .add_axiom(Axiom::DisjointClasses(vec![a, b]))
        .expect("disjoint");

    let report = RlEngine::new(1).saturate(&mut ontology).expect("saturate");

    assert!(!report.clashes.is_empty());
}

/// RL cls-disjoint2: disjoint clash when individual types are equivalent to disjoint classes.
#[test]
fn disjoint_classes_via_equivalence_report_clash() {
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

    assert!(!report.clashes.is_empty());
}
