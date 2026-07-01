use std::collections::HashMap;

use ontologos_core::EntityId;
use ontologos_core::Taxonomy;
use petgraph::Direction;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;

/// Directed hierarchy graph built from a classified taxonomy.
#[derive(Debug, Default)]
pub struct TaxonomyGraph {
    nodes: HashMap<EntityId, NodeIndex>,
    graph: DiGraph<EntityId, ()>,
}

impl TaxonomyGraph {
    #[must_use]
    pub fn from_taxonomy(taxonomy: &Taxonomy) -> Self {
        let mut tg = Self::default();
        for &(sub, sup) in &taxonomy.subsumptions {
            tg.add_edge(sub, sup);
        }
        tg
    }

    fn add_edge(&mut self, sub: EntityId, sup: EntityId) {
        let sub_idx = *self
            .nodes
            .entry(sub)
            .or_insert_with(|| self.graph.add_node(sub));
        let sup_idx = *self
            .nodes
            .entry(sup)
            .or_insert_with(|| self.graph.add_node(sup));
        self.graph.add_edge(sub_idx, sup_idx, ());
    }

    #[must_use]
    pub fn direct_superclasses(&self, class: EntityId) -> Vec<EntityId> {
        let Some(&idx) = self.nodes.get(&class) else {
            return Vec::new();
        };
        self.graph
            .edges_directed(idx, Direction::Outgoing)
            .map(|e| self.graph[e.target()])
            .collect()
    }

    #[must_use]
    pub fn direct_subclasses(&self, class: EntityId) -> Vec<EntityId> {
        let Some(&idx) = self.nodes.get(&class) else {
            return Vec::new();
        };
        self.graph
            .edges_directed(idx, Direction::Incoming)
            .map(|e| self.graph[e.source()])
            .collect()
    }

    #[must_use]
    pub fn is_subsumed(&self, sub: EntityId, sup: EntityId) -> bool {
        if sub == sup {
            return true;
        }
        let Some(&sub_idx) = self.nodes.get(&sub) else {
            return false;
        };
        let Some(&sup_idx) = self.nodes.get(&sup) else {
            return false;
        };
        petgraph::algo::has_path_connecting(&self.graph, sub_idx, sup_idx, None)
    }
}

#[cfg(test)]
mod tests {
    use ontologos_core::{EntityId, Taxonomy};

    use super::*;

    #[test]
    fn graph_finds_transitive_path() {
        let a = EntityId(1);
        let b = EntityId(2);
        let c = EntityId(3);
        let taxonomy = Taxonomy::from_parts(vec![(a, b), (b, c)], vec![], vec![]);
        let g = TaxonomyGraph::from_taxonomy(&taxonomy);
        assert!(g.is_subsumed(a, c));
    }
}
