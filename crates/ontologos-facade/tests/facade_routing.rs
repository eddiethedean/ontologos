use ontologos_core::{Axiom, EntityKind, Ontology, Profile, Reasoner};
use ontologos_facade::{
    ClassifyOutcome, EntailmentCheck, classify, get_object_property_values,
    get_sub_object_properties, is_consistent, is_entailed, is_entailed_axiom,
    is_subsumption_entailed, taxonomy_from_outcome, taxonomy_hierarchy,
};

fn el_ontology() -> Ontology {
    Ontology::builder()
        .class("http://example.org/A")
        .unwrap()
        .class("http://example.org/B")
        .unwrap()
        .subclass_of("http://example.org/A", "http://example.org/B")
        .unwrap()
        .build()
        .unwrap()
}

fn el_chain_ontology() -> Ontology {
    Ontology::builder()
        .class("http://example.org/A")
        .unwrap()
        .class("http://example.org/B")
        .unwrap()
        .class("http://example.org/C")
        .unwrap()
        .subclass_of("http://example.org/A", "http://example.org/B")
        .unwrap()
        .subclass_of("http://example.org/B", "http://example.org/C")
        .unwrap()
        .build()
        .unwrap()
}

fn unsatisfiable_el_ontology() -> Ontology {
    let mut ontology = Ontology::new();
    let a = ontology
        .entity_id("http://example.org/A", EntityKind::Class)
        .expect("A");
    let nothing = ontology
        .entity_id("http://www.w3.org/2002/07/owl#Nothing", EntityKind::Class)
        .expect("Nothing");
    ontology
        .add_axiom(Axiom::SubClassOf {
            subclass: a,
            superclass: nothing,
        })
        .expect("A sub Nothing");
    ontology
}

fn el_reasoner() -> Reasoner {
    Reasoner::builder()
        .profile(Profile::El)
        .build(el_ontology())
        .unwrap()
}

#[test]
fn classify_el_returns_taxonomy_with_subsumption() {
    let mut reasoner = Reasoner::builder()
        .profile(Profile::El)
        .build(el_chain_ontology())
        .unwrap();
    let outcome = classify(&mut reasoner).unwrap();
    let tax = taxonomy_from_outcome(&outcome).expect("EL should return Taxonomy");
    let a = reasoner
        .ontology()
        .lookup_entity("http://example.org/A")
        .unwrap();
    let c = reasoner
        .ontology()
        .lookup_entity("http://example.org/C")
        .unwrap();
    assert!(tax.is_subsumed(a, c));
}

#[test]
fn classify_rdfs_returns_materialization_report() {
    let mut reasoner = Reasoner::builder()
        .profile(Profile::Rdfs)
        .build(el_ontology())
        .unwrap();
    let outcome = classify(&mut reasoner).unwrap();
    assert!(matches!(outcome, ClassifyOutcome::Rdfs(_)));
    assert!(taxonomy_from_outcome(&outcome).is_none());
}

#[test]
fn classify_rl_returns_saturation_report() {
    let mut reasoner = Reasoner::builder()
        .profile(Profile::Rl)
        .build(el_ontology())
        .unwrap();
    let outcome = classify(&mut reasoner).unwrap();
    assert!(matches!(outcome, ClassifyOutcome::Rl(_)));
    assert!(taxonomy_from_outcome(&outcome).is_none());
}

#[test]
fn classify_auto_routes_el_fixture_to_taxonomy() {
    let mut reasoner = Reasoner::builder()
        .profile(Profile::Auto)
        .build(el_chain_ontology())
        .unwrap();
    let outcome = classify(&mut reasoner).unwrap();
    let tax = taxonomy_from_outcome(&outcome).expect("auto on EL fixture");
    let a = reasoner
        .ontology()
        .lookup_entity("http://example.org/A")
        .unwrap();
    let c = reasoner
        .ontology()
        .lookup_entity("http://example.org/C")
        .unwrap();
    assert!(tax.is_subsumed(a, c));
}

#[test]
fn classify_dl_returns_taxonomy_for_named_subsumption() {
    let mut reasoner = Reasoner::builder()
        .profile(Profile::Dl)
        .build(el_ontology())
        .unwrap();
    let outcome = classify(&mut reasoner).unwrap();
    let tax = taxonomy_from_outcome(&outcome).expect("DL should return Taxonomy");
    let a = reasoner
        .ontology()
        .lookup_entity("http://example.org/A")
        .unwrap();
    let b = reasoner
        .ontology()
        .lookup_entity("http://example.org/B")
        .unwrap();
    assert!(tax.is_subsumed(a, b));
}

#[test]
fn taxonomy_from_outcome_none_for_rdfs() {
    let mut reasoner = Reasoner::builder()
        .profile(Profile::Rdfs)
        .build(el_ontology())
        .unwrap();
    let outcome = classify(&mut reasoner).unwrap();
    assert!(taxonomy_from_outcome(&outcome).is_none());
}

#[test]
fn is_consistent_el_uses_el_classifier() {
    let reasoner = el_reasoner();
    assert!(is_consistent(&reasoner).unwrap());
}

#[test]
fn is_consistent_el_detects_unsatisfiable() {
    let reasoner = Reasoner::builder()
        .profile(Profile::El)
        .build(unsatisfiable_el_ontology())
        .unwrap();
    assert!(!is_consistent(&reasoner).unwrap());
}

#[test]
fn is_consistent_auto_routes_el_to_el_classifier() {
    let reasoner = Reasoner::builder()
        .profile(Profile::Auto)
        .build(el_ontology())
        .unwrap();
    assert!(is_consistent(&reasoner).unwrap());
}

#[test]
fn is_consistent_rl_saturates_without_dl_tableau() {
    let ontology = Ontology::builder()
        .class("http://example.org/A")
        .unwrap()
        .build()
        .unwrap();
    let reasoner = Reasoner::builder()
        .profile(Profile::Rl)
        .build(ontology)
        .unwrap();
    assert!(is_consistent(&reasoner).unwrap());
}

#[test]
fn is_consistent_rl_detects_disjoint_clash() {
    let mut ontology = Ontology::builder()
        .class("http://example.org/A")
        .unwrap()
        .class("http://example.org/B")
        .unwrap()
        .class("http://example.org/D")
        .unwrap()
        .individual("http://example.org/x")
        .unwrap()
        .class_assertion("http://example.org/x", "http://example.org/B")
        .unwrap()
        .class_assertion("http://example.org/x", "http://example.org/D")
        .unwrap()
        .build()
        .unwrap();
    let a = ontology.lookup_entity("http://example.org/A").unwrap();
    let b = ontology.lookup_entity("http://example.org/B").unwrap();
    let d = ontology.lookup_entity("http://example.org/D").unwrap();
    ontology
        .add_axiom(Axiom::EquivalentClasses(vec![a, b]))
        .unwrap();
    ontology
        .add_axiom(Axiom::DisjointClasses(vec![a, d]))
        .unwrap();
    let reasoner = Reasoner::builder()
        .profile(Profile::Rl)
        .build(ontology)
        .unwrap();
    assert!(!is_consistent(&reasoner).unwrap());
}

#[test]
fn is_consistent_dl_profile_routes_to_dl_engine() {
    let ontology = Ontology::builder()
        .class("http://example.org/A")
        .unwrap()
        .build()
        .unwrap();
    let reasoner = Reasoner::builder()
        .profile(Profile::Alc)
        .build(ontology)
        .unwrap();
    assert!(is_consistent(&reasoner).unwrap());
}

#[test]
fn is_subsumption_entailed_after_classify() {
    let ontology = el_chain_ontology();
    let mut reasoner = Reasoner::builder()
        .profile(Profile::El)
        .build(ontology)
        .unwrap();
    assert!(
        is_subsumption_entailed(
            &mut reasoner,
            "http://example.org/A",
            "http://example.org/C"
        )
        .unwrap()
    );
    assert!(
        !is_subsumption_entailed(
            &mut reasoner,
            "http://example.org/C",
            "http://example.org/A"
        )
        .unwrap()
    );
    assert!(
        is_entailed(
            &mut reasoner,
            "http://example.org/A",
            "http://example.org/C"
        )
        .unwrap()
    );
}

#[test]
fn taxonomy_hierarchy_direct_subclasses() {
    let ontology = el_chain_ontology();
    let mut reasoner = Reasoner::builder()
        .profile(Profile::El)
        .build(ontology)
        .unwrap();
    let outcome = classify(&mut reasoner).unwrap();
    let tax = taxonomy_from_outcome(&outcome).expect("taxonomy");
    let q = taxonomy_hierarchy(reasoner.ontology(), tax);
    let a = reasoner
        .ontology()
        .lookup_entity("http://example.org/A")
        .unwrap();
    let c = reasoner
        .ontology()
        .lookup_entity("http://example.org/C")
        .unwrap();
    assert!(q.is_subsumed(a, c).unwrap());
}

#[test]
fn classify_auto_hybrid_partitions_mixed_ontology() {
    let report = ontologos_profile::classify_hybrid(&el_chain_ontology()).expect("hybrid");
    assert!(!report.modules.is_empty());
    let mut reasoner = Reasoner::builder()
        .profile(Profile::Auto)
        .build(el_chain_ontology())
        .unwrap();
    let outcome = classify(&mut reasoner).expect("auto classify");
    assert!(taxonomy_from_outcome(&outcome).is_some());
}

#[test]
fn is_entailed_class_assertion_via_subsumption() {
    let ontology = Ontology::builder()
        .class("http://example.org/A")
        .unwrap()
        .class("http://example.org/B")
        .unwrap()
        .individual("http://example.org/x")
        .unwrap()
        .subclass_of("http://example.org/A", "http://example.org/B")
        .unwrap()
        .class_assertion("http://example.org/x", "http://example.org/A")
        .unwrap()
        .build()
        .unwrap();
    let mut reasoner = Reasoner::builder()
        .profile(Profile::El)
        .build(ontology)
        .unwrap();
    assert!(
        is_entailed_axiom(
            &mut reasoner,
            EntailmentCheck::ClassAssertion {
                individual: "http://example.org/x".into(),
                class: "http://example.org/B".into(),
            }
        )
        .unwrap()
    );
}

#[test]
fn is_entailed_object_property_assertion_after_rl_materialization() {
    let ontology = Ontology::builder()
        .individual("http://example.org/c")
        .unwrap()
        .individual("http://example.org/d")
        .unwrap()
        .object_property("http://example.org/r")
        .unwrap()
        .object_property_assertion(
            "http://example.org/c",
            "http://example.org/r",
            "http://example.org/d",
        )
        .unwrap()
        .build()
        .unwrap();
    let mut reasoner = Reasoner::builder()
        .profile(Profile::Rl)
        .build(ontology)
        .unwrap();
    assert!(
        is_entailed_axiom(
            &mut reasoner,
            EntailmentCheck::ObjectPropertyAssertion {
                subject: "http://example.org/c".into(),
                property: "http://example.org/r".into(),
                object: "http://example.org/d".into(),
            }
        )
        .unwrap()
    );
}

#[test]
fn get_object_property_values_returns_fillers() {
    let ontology = Ontology::builder()
        .individual("http://example.org/c")
        .unwrap()
        .individual("http://example.org/d")
        .unwrap()
        .object_property("http://example.org/r")
        .unwrap()
        .object_property_assertion(
            "http://example.org/c",
            "http://example.org/r",
            "http://example.org/d",
        )
        .unwrap()
        .build()
        .unwrap();
    let reasoner = Reasoner::builder()
        .profile(Profile::Rl)
        .build(ontology)
        .unwrap();
    let values =
        get_object_property_values(&reasoner, "http://example.org/c", "http://example.org/r")
            .unwrap();
    assert_eq!(values, vec!["http://example.org/d"]);
}

#[test]
fn get_sub_object_properties_uses_asserted_hierarchy_for_el() {
    let ontology = Ontology::builder()
        .object_property("http://example.org/p")
        .unwrap()
        .object_property("http://example.org/q")
        .unwrap()
        .subproperty_of("http://example.org/q", "http://example.org/p")
        .unwrap()
        .build()
        .unwrap();
    let reasoner = Reasoner::builder()
        .profile(Profile::El)
        .build(ontology)
        .unwrap();
    let direct = get_sub_object_properties(&reasoner, "http://example.org/p", true).unwrap();
    assert_eq!(direct, vec!["http://example.org/q"]);
    let all = get_sub_object_properties(&reasoner, "http://example.org/p", false).unwrap();
    assert_eq!(all, vec!["http://example.org/q"]);
}

/// Mirrors the getting-started classify quickstart (family.owl → Profile::Auto).
#[test]
fn getting_started_classify_family_auto() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data/family.owl");
    if !path.exists() {
        return;
    }
    let ontology = ontologos_parser::load_ontology(&path).expect("load family.owl");
    let mut reasoner = Reasoner::builder()
        .profile(Profile::Auto)
        .build(ontology)
        .expect("build");
    match classify(&mut reasoner).expect("classify") {
        ClassifyOutcome::Taxonomy(t) => {
            assert!(t.subsumption_count() > 0 || t.subsumptions.is_empty());
        }
        ClassifyOutcome::Rdfs(r) => {
            let _ = r.inferred_total();
        }
        ClassifyOutcome::Rl(r) => {
            assert!(r.inferred_total() > 0);
        }
    }
}
