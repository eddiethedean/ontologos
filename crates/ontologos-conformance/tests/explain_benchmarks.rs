//! v0.6 explanation exit criteria with dependency-first adapters.
//!
//! EL explanations use in-house completion traces; RDFS/RL traces remain
//! empty until reasonable exposes rule-level diagnostics.

use ontologos_core::{EntityKind, Ontology, Profile, Reasoner, ReasonerConfig};
use ontologos_el::ElClassifier;
use ontologos_explain::{build_proof_graph, collect_trace, explain_el, explain_rdfs, explain_rl};
use ontologos_parser::load_ontology;
use ontologos_rdfs::RdfsEngine;
use ontologos_rl::RlEngine;

fn family_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data/family.owl")
}

fn pizza_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data/pizza.owl")
}

fn assert_valid_graph(ontology: &Ontology, graph: &ontologos_explain::ProofGraph) {
    assert!(!graph.nodes.is_empty(), "expected non-empty proof graph");
    assert!(graph.is_acyclic(), "proof graph must be acyclic");
    for node in &graph.nodes {
        if let Some(id) = node.conclusion_axiom {
            ontology
                .axiom(id)
                .unwrap_or_else(|_| panic!("valid conclusion axiom id {}", id.0));
        }
        for premise in &node.premises {
            assert!(
                (premise.0 as usize) < graph.nodes.len(),
                "premise node id must exist"
            );
        }
    }
}

fn reasoner_with_profile(ontology: Ontology, profile: Profile) -> Reasoner {
    Reasoner::builder()
        .profile(profile)
        .config(ReasonerConfig {
            explanations: true,
            ..ReasonerConfig::default()
        })
        .build(ontology)
        .expect("reasoner")
}

#[test]
fn rdfs_family_materialization_and_graph() {
    let path = family_path();
    if !path.exists() {
        eprintln!("skip: missing {}", path.display());
        return;
    }
    let mut ontology = load_ontology(&path).expect("load family");
    let before = ontology.axiom_count();
    let graph = explain_rdfs(&mut ontology).expect("rdfs explain");
    assert_valid_graph(&ontology, &graph);
    assert!(
        ontology.axiom_count() > before,
        "reasonable adapter should materialize family"
    );
}

#[test]
fn rl_family_saturation_and_graph() {
    let path = family_path();
    if !path.exists() {
        return;
    }
    let mut ontology = load_ontology(&path).expect("load family");
    let before = ontology.axiom_count();
    let graph = explain_rl(&mut ontology).expect("rl explain");
    assert_valid_graph(&ontology, &graph);
    assert!(ontology.axiom_count() > before, "RL saturation expected");
}

#[test]
fn el_chain_subsumption_via_completion() {
    let mut ontology = Ontology::new();
    let a = ontology
        .entity_id("http://ex.org/A", EntityKind::Class)
        .unwrap();
    let b = ontology
        .entity_id("http://ex.org/B", EntityKind::Class)
        .unwrap();
    let c = ontology
        .entity_id("http://ex.org/C", EntityKind::Class)
        .unwrap();
    ontology
        .add_axiom(ontologos_core::Axiom::SubClassOf {
            subclass: a,
            superclass: b,
        })
        .unwrap();
    ontology
        .add_axiom(ontologos_core::Axiom::SubClassOf {
            subclass: b,
            superclass: c,
        })
        .unwrap();

    let taxonomy = ElClassifier::new().classify(&ontology).expect("classify");
    assert!(taxonomy.is_subsumed(a, c));

    let graph = explain_el(&ontology).expect("el explain");
    assert_valid_graph(&ontology, &graph);

    let mut reasoner = reasoner_with_profile(ontology, Profile::El);
    let trace = collect_trace(&mut reasoner).expect("trace");
    let full = build_proof_graph(reasoner.ontology(), &trace).expect("graph");
    assert_valid_graph(reasoner.ontology(), &full);
}

#[test]
fn el_pizza_subsumption_explanation() {
    let path = pizza_path();
    if !path.exists() {
        eprintln!("skip: run ./benchmarks/scripts/download.sh for pizza.owl");
        return;
    }
    let ontology = load_ontology(&path).expect("load pizza");
    let taxonomy = ElClassifier::new().classify(&ontology).expect("classify");
    assert!(taxonomy.subsumption_count() > 0);
    let graph = explain_el(&ontology).expect("el explain");
    assert_valid_graph(&ontology, &graph);
}

#[test]
fn rdfs_engine_smoke() {
    let path = family_path();
    if !path.exists() {
        return;
    }
    let mut ontology = load_ontology(&path).expect("load");
    RdfsEngine::new()
        .materialize(&mut ontology)
        .expect("materialize");
}

#[test]
fn rl_engine_smoke() {
    let path = family_path();
    if !path.exists() {
        return;
    }
    let mut ontology = load_ontology(&path).expect("load");
    RlEngine::new(1).saturate(&mut ontology).expect("saturate");
}

#[test]
fn el_existential_subsumption() {
    let mut ontology = Ontology::new();
    let a = ontology
        .entity_id("http://ex.org/A", EntityKind::Class)
        .unwrap();
    let b = ontology
        .entity_id("http://ex.org/B", EntityKind::Class)
        .unwrap();
    let c = ontology
        .entity_id("http://ex.org/C", EntityKind::Class)
        .unwrap();
    let r = ontology
        .entity_id("http://ex.org/r", EntityKind::ObjectProperty)
        .unwrap();
    ontology
        .add_axiom(ontologos_core::Axiom::SubClassOfExistential {
            subclass: a,
            property: r,
            filler: b,
        })
        .unwrap();
    ontology
        .add_axiom(ontologos_core::Axiom::SubClassOf {
            subclass: b,
            superclass: c,
        })
        .unwrap();

    // In-house EL: B ⊑ C via subsumption transitivity; A ⊑ ∃r.B is asserted, not A ⊑ C directly.
    let taxonomy = ElClassifier::new().classify(&ontology).expect("classify");
    assert!(taxonomy.is_subsumed(b, c));
}

#[test]
fn ten_benchmark_inferences_across_engines() {
    let path = family_path();
    if !path.exists() {
        return;
    }
    let mut rdfs_ont = load_ontology(&path).expect("load");
    let rdfs_report = RdfsEngine::new().materialize(&mut rdfs_ont).expect("rdfs");
    let mut rl_ont = load_ontology(&path).expect("load");
    let rl_report = RlEngine::new(1).saturate(&mut rl_ont).expect("rl");
    let el_ont = load_ontology(&path).expect("load");
    let el_tax = ElClassifier::new().classify(&el_ont).expect("el");

    let total =
        rdfs_report.inferred_total() + rl_report.inferred_total() + el_tax.subsumption_count();
    assert!(
        total >= 10,
        "expected >=10 combined inferences across engines, got {total}"
    );
}
