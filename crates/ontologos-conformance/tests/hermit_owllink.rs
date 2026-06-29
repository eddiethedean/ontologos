//! Hand-written ports for HermiT `OWLLinkTest` smoke and entailment checks.
//!
//! Source: `HermiT/project/test/org/semanticweb/HermiT/reasoner/OWLLinkTest.java`

use ontologos_conformance::{hermit_test_path, vendored_hermit_test_path};
use ontologos_core::{Axiom, DlAxiom, Ontology};
use ontologos_dl::is_consistent;
use ontologos_parser::load_ontology;
use std::path::PathBuf;

const FAMILIES_NS: &str = "http://example.com/owl/families/";

fn fixture_path(name: &str) -> PathBuf {
    let rel = format!("reasoner/res/{name}");
    vendored_hermit_test_path(&rel)
        .or_else(|| hermit_test_path(&rel))
        .unwrap_or_else(|| panic!("missing OWLLink fixture {name}"))
}

fn primer_fragment_ofn() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_owllinktest_primer_fragment.ofn",
    )
}

fn families_iri(local: &str) -> String {
    format!("{FAMILIES_NS}{local}")
}

fn load_owllink_primer() -> Ontology {
    load_ontology(&fixture_path("primer.owl")).expect("primer.owl with families import")
}

fn load_owllink_primer_fragment() -> Ontology {
    load_ontology(&primer_fragment_ofn()).expect("owllink primer fragment")
}

fn has_disjoint_object_properties(ontology: &Ontology, left: &str, right: &str) -> bool {
    let Some(left) = ontology.lookup_entity(left) else {
        return false;
    };
    let Some(right) = ontology.lookup_entity(right) else {
        return false;
    };
    ontology.dl().axioms().any(|axiom| {
        let DlAxiom::DisjointObjectProperties(props) = axiom else {
            return false;
        };
        props.contains(&left) && props.contains(&right)
    })
}

fn has_disjoint_classes(ontology: &Ontology, left: &str, right: &str) -> bool {
    let Some(left) = ontology.lookup_entity(left) else {
        return false;
    };
    let Some(right) = ontology.lookup_entity(right) else {
        return false;
    };
    ontology.axioms().iter().any(|(_, axiom)| {
        matches!(
            axiom,
            Axiom::DisjointClasses(classes)
                if classes.contains(&left) && classes.contains(&right)
        )
    })
}

/// `OWLLinkTest.testInverses` / `testSuccessiveCalls` — primer corpus loads (Java smoke only).
#[test]
fn owllink_primer_smoke() {
    let ontology = load_owllink_primer();
    assert!(ontology.entity_count() > 0);
    assert!(
        ontology.lookup_entity(&families_iri("hasParent")).is_some(),
        "expected hasParent from primer corpus"
    );
}

/// `OWLLinkTest.testDisjointProperties` — hasParent and hasSpouse are disjoint in primer.
#[test]
fn owllink_disjoint_properties_has_parent_spouse() {
    let ontology = load_owllink_primer();
    assert!(has_disjoint_object_properties(
        &ontology,
        &families_iri("hasParent"),
        &families_iri("hasSpouse"),
    ));
}

/// `OWLLinkTest.testDisjointClasses` — Father is disjoint with Mother in primer.
#[test]
fn owllink_disjoint_classes_father_mother() {
    let ontology = load_owllink_primer();
    assert!(has_disjoint_classes(
        &ontology,
        &families_iri("Father"),
        &families_iri("Mother"),
    ));
}

/// Minimal OFN fragment remains a fast regression when primer RDF changes.
#[test]
fn owllink_primer_fragment_disjoint_axioms() {
    let ontology = load_owllink_primer_fragment();
    assert!(has_disjoint_object_properties(
        &ontology,
        &families_iri("hasParent"),
        &families_iri("hasSpouse"),
    ));
    assert!(has_disjoint_classes(
        &ontology,
        &families_iri("Father"),
        &families_iri("Mother"),
    ));
}

/// `OWLLinkTest.testObjectProperties` — declared property does not crash consistency check.
#[test]
fn owllink_object_properties_declaration_smoke() {
    let ontology = Ontology::builder()
        .object_property(&families_iri("hasParent"))
        .expect("hasParent")
        .build()
        .expect("build");
    assert!(is_consistent(&ontology).expect("consistent"));
}

#[test]
fn owllink_vendored_fixtures_present() {
    for name in ["primer.owl", "families.owl"] {
        assert!(
            fixture_path(name).is_file(),
            "vendored OWLLink fixture {name} should exist for future full-corpus work"
        );
    }
}

const IYOUIT_AGENT_NS: &str = "http://www.iyouit.eu/agent.owl#";

/// `OWLLinkTest.testBobTestAandB` — direct/all subproperties of `knows` in IYOUIT agent.owl.
#[test]
#[ignore = "diagnostic: dump knows subproperty IRIs and entailment checks"]
fn owllink_bob_knows_subproperties_diag() {
    use ontologos_dl::RolePropertyQueryContext;
    use ontologos_core::RoleExpr;

    let path = fixture_path("OWLLink/agent.owl");
    let ontology = load_ontology(&path).expect("load agent.owl");
    let knows = RoleExpr::Atomic(
        ontology
            .lookup_entity(&format!("{IYOUIT_AGENT_NS}knows"))
            .expect("knows property"),
    );
    let query = RolePropertyQueryContext::prepare(&ontology).expect("prepare");
    let direct = query
        .sub_object_property_expressions(&knows, true)
        .expect("direct");
    let all = query.sub_object_property_expressions(&knows, false).expect("all");
    let mut direct_iris: Vec<_> = direct
        .iter()
        .filter_map(|r| role_iri(&ontology, r))
        .collect();
    direct_iris.sort();
    let mut all_iris: Vec<_> = all
        .iter()
        .filter_map(|r| role_iri(&ontology, r))
        .collect();
    all_iris.sort();
    eprintln!("direct ({}):", direct_iris.len());
    for iri in &direct_iris {
        eprintln!("  {iri}");
    }
    eprintln!("all ({}):", all_iris.len());
    for iri in &all_iris {
        eprintln!("  {iri}");
    }

    let relation_id = ontology
        .lookup_entity(&format!("{IYOUIT_AGENT_NS}relation"))
        .expect("relation");
    let relation = RoleExpr::Atomic(relation_id);
    let registered = RoleExpr::Atomic(
        ontology
            .lookup_entity(&format!("{IYOUIT_AGENT_NS}registered"))
            .expect("registered"),
    );
    let wants = RoleExpr::Atomic(
        ontology
            .lookup_entity(&format!("{IYOUIT_AGENT_NS}wants_to_know"))
            .expect("wants_to_know"),
    );
    eprintln!("relation in all: {}", all.contains(&relation));
    eprintln!(
        "inv(relation) in all: {}",
        all.contains(&RoleExpr::Inverse(relation_id))
    );
    eprintln!("registered in all: {}", all.contains(&registered));
    eprintln!("wants_to_know in all: {}", all.contains(&wants));
}

fn role_iri(ontology: &ontologos_core::Ontology, role: &ontologos_core::RoleExpr) -> Option<String> {
    use ontologos_core::RoleExpr;
    match role {
        RoleExpr::Atomic(id) => ontology
            .entity(*id)
            .ok()
            .and_then(|r| ontology.resolve_iri(r.iri).ok().map(str::to_string)),
        RoleExpr::Inverse(id) => ontology.entity(*id).ok().and_then(|r| {
            ontology
                .resolve_iri(r.iri)
                .ok()
                .map(|iri| format!("inv({iri})"))
        }),
    }
}

/// `OWLLinkTest.testBobTestAandB` — direct/all subproperties of `knows` in IYOUIT agent.owl.
#[test]
fn owllink_bob_knows_subproperties() {
    use ontologos_dl::RolePropertyQueryContext;
    use ontologos_core::RoleExpr;

    let path = fixture_path("OWLLink/agent.owl");
    let ontology = load_ontology(&path).expect("load agent.owl");
    let knows = RoleExpr::Atomic(
        ontology
            .lookup_entity(&format!("{IYOUIT_AGENT_NS}knows"))
            .expect("knows property"),
    );
    let query = RolePropertyQueryContext::prepare(&ontology).expect("prepare");
    let direct = query
        .sub_object_property_expressions(&knows, true)
        .expect("direct");
    let all = query.sub_object_property_expressions(&knows, false).expect("all");
    assert_eq!(
        direct.len(),
        20,
        "HermiT OWLLink Bob test A expects 20 direct subproperties of knows"
    );
    assert_eq!(
        all.len(),
        101,
        "HermiT OWLLink Bob test B expects 101 subproperties of knows"
    );
}
