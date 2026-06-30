//! Minimal HermiT-style description graph (stub for graph.* internal tests).

use std::collections::HashSet;

/// Stable graph identity for extension-table tuples.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DescriptionGraphId(pub u32);

/// Directed edge labelled by an atomic role name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DescriptionGraphEdge {
    /// Role IRI/local name.
    pub role: &'static str,
    /// Source vertex index.
    pub from: usize,
    /// Target vertex index.
    pub to: usize,
}

/// HermiT `DescriptionGraph` — vertices carry atomic concept names.
#[derive(Debug, Clone)]
pub struct DescriptionGraph {
    id: DescriptionGraphId,
    vertex_concepts: Vec<&'static str>,
    edges: Vec<DescriptionGraphEdge>,
    start_concepts: HashSet<&'static str>,
}

impl DescriptionGraph {
    /// Construct a description graph (HermiT `DescriptionGraph` constructor).
    #[must_use]
    pub fn new(
        id: DescriptionGraphId,
        vertex_concepts: Vec<&'static str>,
        edges: Vec<DescriptionGraphEdge>,
        start_concepts: Vec<&'static str>,
    ) -> Self {
        Self {
            id,
            vertex_concepts,
            edges,
            start_concepts: start_concepts.into_iter().collect(),
        }
    }

    /// Graph identifier used in 4-ary extension tuples.
    #[must_use]
    pub fn id(&self) -> DescriptionGraphId {
        self.id
    }

    /// Number of vertices (extension tuple arity is `vertices + 1`).
    #[must_use]
    pub fn number_of_vertices(&self) -> usize {
        self.vertex_concepts.len()
    }

    /// Concept at vertex `index`.
    #[must_use]
    pub fn vertex_concept(&self, index: usize) -> Option<&'static str> {
        self.vertex_concepts.get(index).copied()
    }

    /// Outgoing edges from `from`.
    pub fn outgoing_edges(&self, from: usize) -> impl Iterator<Item = &DescriptionGraphEdge> {
        self.edges.iter().filter(move |edge| edge.from == from)
    }

    /// Whether `concept` is a start concept for this graph.
    #[must_use]
    pub fn is_start_concept(&self, concept: &str) -> bool {
        self.start_concepts.contains(concept)
    }

    /// HermiT test helper `G(...)`.
    #[must_use]
    pub fn test_graph(
        id: u32,
        vertex_concepts: Vec<&'static str>,
        edges: Vec<DescriptionGraphEdge>,
        start_concepts: Vec<&'static str>,
    ) -> Self {
        Self::new(DescriptionGraphId(id), vertex_concepts, edges, start_concepts)
    }

    /// HermiT test helper `E(role, from, to)`.
    #[must_use]
    pub fn edge(role: &'static str, from: usize, to: usize) -> DescriptionGraphEdge {
        DescriptionGraphEdge { role, from, to }
    }
}
