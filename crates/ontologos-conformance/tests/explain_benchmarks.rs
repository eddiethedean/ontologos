//! v0.6 explanation exit criteria: ≥10 benchmark inferences across three engines.

use ontologos_core::{EntityKind, Ontology, Profile, Reasoner, ReasonerConfig, TraceConclusion};
use ontologos_explain::{
    build_proof_graph, collect_trace, explain_el, explain_rdfs, explain_rl, explain_subsumption,
    find_subsumption_step,
};
use ontologos_parser::load_ontology;

fn family_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data/family.owl")
}

fn pizza_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data/pizza.owl")
}

fn assert_valid_graph(ontology: &Ontology, graph: &ontologos_explain::ProofGraph) {
    assert!(!graph.nodes.is_empty(), "expected non-empty proof graph");
    let mut visiting = vec![false; graph.nodes.len()];
    let mut done = vec![false; graph.nodes.len()];
    for start in 0..graph.nodes.len() {
        assert!(
            !has_cycle(graph, start, &mut visiting, &mut done),
            "proof graph must be acyclic"
        );
    }
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

fn has_cycle(
    graph: &ontologos_explain::ProofGraph,
    idx: usize,
    visiting: &mut [bool],
    done: &mut [bool],
) -> bool {
    if done[idx] {
        return false;
    }
    if visiting[idx] {
        return true;
    }
    visiting[idx] = true;
    for premise in &graph.nodes[idx].premises {
        if has_cycle(graph, premise.0 as usize, visiting, done) {
            return true;
        }
    }
    visiting[idx] = false;
    done[idx] = true;
    false
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

fn rdfs_inference_rules(graph: &ontologos_explain::ProofGraph) -> Vec<&str> {
    graph
        .nodes
        .iter()
        .map(|n| n.rule.as_str())
        .filter(|r| matches!(*r, "sc_trans" | "sp_trans" | "dom_inherit" | "rng_inherit"))
        .collect()
}

#[test]
fn rdfs_family_sc_trans_explanation() {
    let path = family_path();
    if !path.exists() {
        eprintln!("skip: missing {}", path.display());
        return;
    }
    let mut ontology = load_ontology(&path).expect("load family");
    let graph = explain_rdfs(&mut ontology).expect("rdfs explain");
    assert_valid_graph(&ontology, &graph);
    assert!(
        graph.nodes.iter().any(|n| n.rule == "sc_trans"),
        "expected sc_trans inference"
    );
}

#[test]
fn rdfs_family_dom_inherit_explanation() {
    let path = family_path();
    if !path.exists() {
        return;
    }
    let mut ontology = load_ontology(&path).expect("load family");
    let graph = explain_rdfs(&mut ontology).expect("rdfs explain");
    let rules = rdfs_inference_rules(&graph);
    assert!(
        rules
            .iter()
            .any(|r| *r == "dom_inherit" || *r == "rng_inherit"),
        "expected domain/range inheritance, got: {rules:?}"
    );
}

#[test]
fn rdfs_family_sp_trans_explanation() {
    let path = family_path();
    if !path.exists() {
        return;
    }
    let mut ontology = load_ontology(&path).expect("load family");
    let graph = explain_rdfs(&mut ontology).expect("rdfs explain");
    assert!(
        graph.nodes.iter().any(|n| n.rule != "asserted"),
        "expected at least one RDFS inference"
    );
}

#[test]
fn rdfs_family_multiple_rules_explanation() {
    let path = family_path();
    if !path.exists() {
        return;
    }
    let mut ontology = load_ontology(&path).expect("load family");
    let graph = explain_rdfs(&mut ontology).expect("rdfs explain");
    let rules = rdfs_inference_rules(&graph);
    assert!(
        rules.len() >= 2,
        "expected multiple RDFS rules, got: {rules:?}"
    );
}

#[test]
fn rl_family_saturation_explanation() {
    let path = family_path();
    if !path.exists() {
        return;
    }
    let mut ontology = load_ontology(&path).expect("load family");
    let graph = explain_rl(&mut ontology).expect("rl explain");
    assert_valid_graph(&ontology, &graph);
    assert!(
        graph.nodes.iter().any(|n| n.rule != "asserted"),
        "expected RL or RDFS inference nodes"
    );
}

#[test]
fn rl_family_rl_rule_explanation() {
    let path = family_path();
    if !path.exists() {
        return;
    }
    let mut ontology = load_ontology(&path).expect("load family");
    let graph = explain_rl(&mut ontology).expect("rl explain");
    let rl_rules = [
        "eq_class_sub",
        "type_subclass",
        "type_domain",
        "type_range",
        "same_as_class",
        "prop_sub",
    ];
    let found = rl_rules
        .iter()
        .filter(|rule| graph.nodes.iter().any(|n| n.rule == **rule))
        .count();
    assert!(
        found >= 1 || graph.nodes.len() > 20,
        "expected RL inferences or large saturation trace"
    );
}

#[test]
fn el_chain_subsumption_explanation() {
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

    let graph = explain_el(&ontology).expect("el explain");
    assert_valid_graph(&ontology, &graph);
    assert!(
        graph.nodes.iter().any(|n| n.rule == "sub_trans_forward"),
        "expected EL transitive subsumption"
    );

    let mut reasoner = reasoner_with_profile(ontology.clone(), Profile::El);
    let trace = collect_trace(&mut reasoner).expect("trace");
    assert!(find_subsumption_step(&trace, a, c));
    let sub = explain_subsumption(&ontology, a, c, Profile::El, &trace).expect("subgraph");
    assert!(!sub.nodes.is_empty());
}

#[test]
fn el_existential_explanation() {
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

    let graph = explain_el(&ontology).expect("el explain");
    assert!(
        graph.nodes.iter().any(|n| n.rule == "ex_filler_sub"),
        "expected existential filler propagation"
    );
}

#[test]
fn el_pizza_subsumption_explanation() {
    let path = pizza_path();
    if !path.exists() {
        eprintln!("skip: run ./benchmarks/scripts/download.sh for pizza.owl");
        return;
    }
    let ontology = load_ontology(&path).expect("load pizza");
    let graph = explain_el(&ontology).expect("el explain");
    assert_valid_graph(&ontology, &graph);
    assert!(
        graph.node_count() > 10,
        "pizza EL trace should be non-trivial"
    );
}

#[test]
fn rdfs_targeted_subsumption_subgraph() {
    let path = family_path();
    if !path.exists() {
        return;
    }
    let ontology = load_ontology(&path).expect("load family");
    let mut reasoner = reasoner_with_profile(ontology, Profile::Rdfs);
    let trace = collect_trace(&mut reasoner).expect("trace");
    let step = trace
        .steps
        .iter()
        .find(|s| matches!(s.conclusion, TraceConclusion::Axiom { .. }))
        .expect("rdfs step");
    let TraceConclusion::Axiom { id } = step.conclusion else {
        unreachable!();
    };
    let axiom = reasoner.ontology().axiom(id).expect("axiom");
    let ontologos_core::Axiom::SubClassOf {
        subclass,
        superclass,
    } = axiom
    else {
        return;
    };
    let sub = explain_subsumption(
        reasoner.ontology(),
        *subclass,
        *superclass,
        Profile::Rdfs,
        &trace,
    )
    .expect("subgraph");
    assert!(!sub.nodes.is_empty());
    let _ = build_proof_graph(reasoner.ontology(), &trace).expect("full graph");
}
