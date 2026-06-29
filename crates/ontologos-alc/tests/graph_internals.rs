//! HermiT `graph.GraphTest` engine-internal tests — inventory (Tier B3).
//!
//! HermiT's DescriptionGraph is constructed inside Tableau/DescriptionGraphManager;
//! full ports require the same extension-manager surface as `tableau.*` tests.

const GRAPH_INTERNAL_IDS: &[&str] = &[
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
            "{id} should remain internal until DescriptionGraph port"
        );
    }
    assert_eq!(GRAPH_INTERNAL_IDS.len(), 3);
}
