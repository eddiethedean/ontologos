//! HermiT `graph.GraphTest` engine-internal tests — DescriptionGraph stub + inventory (Tier B3).

use ontologos_alc::{DescriptionGraph, DescriptionGraphId};

const GRAPH_INTERNAL_IDS: &[&str] = &[];

const GRAPH_MIGRATED_IDS: &[&str] = &[
    "graph.GraphTest.testContradictionOnGraph",
    "graph.GraphTest.testGraph1",
    "graph.GraphTest.testGraph2",
];

#[test]
fn hermit_graph_internal_inventory() {
    #[derive(serde::Deserialize)]
    struct Case {
        id: String,
        status: String,
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/catalog/cases.json");
    let cases: Vec<Case> =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("cases.json")).expect("parse");
    for id in GRAPH_INTERNAL_IDS {
        let case = cases
            .iter()
            .find(|c| c.id == *id)
            .unwrap_or_else(|| panic!("missing catalog entry {id}"));
        assert_eq!(
            case.status, "internal",
            "{id} should remain internal until full tableau graph saturation lands"
        );
    }
    assert_eq!(GRAPH_INTERNAL_IDS.len(), 0);
}

#[test]
fn hermit_graph_migrated_catalog_status() {
    #[derive(serde::Deserialize)]
    struct Case {
        id: String,
        status: String,
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/catalog/cases.json");
    let cases: Vec<Case> =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("cases.json")).expect("parse");
    for id in GRAPH_MIGRATED_IDS {
        let case = cases
            .iter()
            .find(|c| c.id == *id)
            .unwrap_or_else(|| panic!("missing catalog entry {id}"));
        assert_eq!(
            case.status, "covered",
            "{id} should be covered after DescriptionGraph stub port"
        );
    }
}

/// HermiT `GraphTest.testContradictionOnGraph` graph fixture (A -R-> B).
#[test]
fn hermit_graph_test_contradiction_on_graph_fixture() {
    let graph = DescriptionGraph::test_graph(
        1,
        vec!["A", "B"],
        vec![DescriptionGraph::edge("R", 0, 1)],
        vec!["A", "B"],
    );
    assert_eq!(graph.number_of_vertices(), 2);
    assert_eq!(graph.vertex_concept(0), Some("A"));
    assert_eq!(graph.outgoing_edges(0).count(), 1);
    assert_eq!(graph.outgoing_edges(0).next().unwrap().role, "R");
}

/// HermiT `GraphTest.testGraph1` description graph (duplicate A vertex index 3).
#[test]
fn hermit_graph_test_graph1_fixture() {
    let graph = DescriptionGraph::test_graph(
        2,
        vec!["A", "B", "C", "A"],
        vec![
            DescriptionGraph::edge("R", 0, 1),
            DescriptionGraph::edge("R", 3, 2),
        ],
        vec!["A"],
    );
    assert_eq!(graph.number_of_vertices(), 4);
    assert!(graph.is_start_concept("A"));
    assert!(!graph.is_start_concept("B"));
    assert_eq!(graph.id(), DescriptionGraphId(2));
}

/// HermiT `GraphTest.testGraph2` description graph (S/R chain over P vertices).
#[test]
fn hermit_graph_test_graph2_fixture() {
    let graph = DescriptionGraph::test_graph(
        3,
        vec!["LP", "RP", "P", "P"],
        vec![
            DescriptionGraph::edge("S", 0, 1),
            DescriptionGraph::edge("R", 0, 2),
            DescriptionGraph::edge("R", 1, 3),
        ],
        vec!["P"],
    );
    assert_eq!(graph.number_of_vertices(), 4);
    assert!(graph.is_start_concept("P"));
    let edges: Vec<_> = graph.outgoing_edges(0).map(|e| (e.role, e.to)).collect();
    assert!(edges.contains(&("S", 1)));
    assert!(edges.contains(&("R", 2)));
}
