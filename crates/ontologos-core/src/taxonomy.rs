use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::entity::EntityId;

/// Adjacency index for subsumption reachability queries.
#[derive(Debug, Clone, Default)]
struct SubsumptionGraph {
    direct_supers: HashMap<EntityId, HashSet<EntityId>>,
    edges: HashSet<(EntityId, EntityId)>,
}

/// Cached subsumption reachability for batch taxonomy queries.
#[derive(Debug, Clone)]
pub struct ReachabilityIndex(SubsumptionGraph);

impl ReachabilityIndex {
    /// Build an index from the current subsumption edges.
    #[must_use]
    pub fn new(taxonomy: &Taxonomy) -> Self {
        Self(SubsumptionGraph::from_edges(&taxonomy.subsumptions))
    }

    /// Whether `sub` is subsumed by `sup` (direct or indirect).
    #[must_use]
    pub fn is_subsumed(&self, sub: EntityId, sup: EntityId) -> bool {
        self.0.reachable(sub, sup)
    }

    /// Record a new direct edge for incremental enrichment passes.
    pub fn insert_edge(&mut self, sub: EntityId, sup: EntityId) -> bool {
        if sub == sup {
            return false;
        }
        if !self.0.edges.insert((sub, sup)) {
            return false;
        }
        self.0.direct_supers.entry(sub).or_default().insert(sup);
        true
    }
}

impl SubsumptionGraph {
    fn from_edges(edges: &[(EntityId, EntityId)]) -> Self {
        let mut graph = Self::default();
        for &(sub, sup) in edges {
            graph.insert(sub, sup);
        }
        graph
    }

    fn insert(&mut self, sub: EntityId, sup: EntityId) {
        self.edges.insert((sub, sup));
        self.direct_supers.entry(sub).or_default().insert(sup);
    }

    fn reachable(&self, from: EntityId, to: EntityId) -> bool {
        if from == to {
            return true;
        }
        let Some(supers) = self.direct_supers.get(&from) else {
            return false;
        };
        let mut stack: Vec<EntityId> = supers.iter().copied().collect();
        let mut seen = HashSet::new();
        while let Some(current) = stack.pop() {
            if current == to {
                return true;
            }
            if !seen.insert(current) {
                continue;
            }
            if let Some(next) = self.direct_supers.get(&current) {
                stack.extend(next.iter().copied());
            }
        }
        false
    }
}

/// Extracted class taxonomy from a classification run.
#[derive(Debug, Serialize, Deserialize)]
pub struct Taxonomy {
    /// Direct subsumptions `(subclass, superclass)` after transitive reduction.
    pub subsumptions: Vec<(EntityId, EntityId)>,
    /// Equivalence class clusters (each vec has ≥ 2 members when from explicit axioms).
    pub equivalences: Vec<Vec<EntityId>>,
    /// Classes inferred equivalent to `owl:Nothing` / ⊥.
    pub unsatisfiable: Vec<EntityId>,
    #[serde(skip)]
    graph_cache: RefCell<Option<SubsumptionGraph>>,
}

impl Default for Taxonomy {
    fn default() -> Self {
        Self {
            subsumptions: Vec::new(),
            equivalences: Vec::new(),
            unsatisfiable: Vec::new(),
            graph_cache: RefCell::new(None),
        }
    }
}

impl PartialEq for Taxonomy {
    fn eq(&self, other: &Self) -> bool {
        self.subsumptions == other.subsumptions
            && self.equivalences == other.equivalences
            && self.unsatisfiable == other.unsatisfiable
    }
}

impl Eq for Taxonomy {}

impl Clone for Taxonomy {
    fn clone(&self) -> Self {
        Self {
            subsumptions: self.subsumptions.clone(),
            equivalences: self.equivalences.clone(),
            unsatisfiable: self.unsatisfiable.clone(),
            graph_cache: RefCell::new(None),
        }
    }
}

impl Taxonomy {
    /// Build a taxonomy from classification outputs.
    #[must_use]
    pub fn from_parts(
        subsumptions: Vec<(EntityId, EntityId)>,
        equivalences: Vec<Vec<EntityId>>,
        unsatisfiable: Vec<EntityId>,
    ) -> Self {
        Self {
            subsumptions,
            equivalences,
            unsatisfiable,
            ..Self::default()
        }
    }

    /// Number of direct subsumption edges.
    #[must_use]
    pub fn subsumption_count(&self) -> usize {
        self.subsumptions.len()
    }

    fn invalidate_graph_cache(&self) {
        *self.graph_cache.borrow_mut() = None;
    }

    fn graph(&self) -> std::cell::Ref<'_, SubsumptionGraph> {
        if self.graph_cache.borrow().is_none() {
            *self.graph_cache.borrow_mut() = Some(SubsumptionGraph::from_edges(&self.subsumptions));
        }
        std::cell::Ref::map(self.graph_cache.borrow(), |c: &Option<SubsumptionGraph>| {
            c.as_ref().expect("graph cache")
        })
    }

    /// Whether a direct subsumption edge `(sub, sup)` is present.
    #[must_use]
    pub fn contains_edge(&self, sub: EntityId, sup: EntityId) -> bool {
        self.graph().edges.contains(&(sub, sup))
    }

    /// Insert a subsumption edge if absent; returns whether a new edge was added.
    pub fn add_subsumption(&mut self, sub: EntityId, sup: EntityId) -> bool {
        if sub == sup || self.contains_edge(sub, sup) {
            return false;
        }
        self.subsumptions.push((sub, sup));
        self.invalidate_graph_cache();
        true
    }

    /// Retain subsumption edges matching `predicate`.
    pub fn retain_subsumptions<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&(EntityId, EntityId)) -> bool,
    {
        self.subsumptions.retain(|edge| predicate(edge));
        self.invalidate_graph_cache();
    }

    /// Direct superclasses of `class` in the reduced taxonomy.
    #[must_use]
    pub fn direct_superclasses(&self, class: EntityId) -> Vec<EntityId> {
        self.graph()
            .direct_supers
            .get(&class)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Direct subclasses of `class` in the reduced taxonomy.
    #[must_use]
    pub fn direct_subclasses(&self, class: EntityId) -> Vec<EntityId> {
        let graph = self.graph();
        let mut out = Vec::new();
        for (&sub, supers) in &graph.direct_supers {
            if supers.contains(&class) {
                out.push(sub);
            }
        }
        out
    }

    /// Whether `sub` is subsumed by `sup` (direct or indirect) in this taxonomy.
    #[must_use]
    pub fn is_subsumed(&self, sub: EntityId, sup: EntityId) -> bool {
        if sub == sup {
            return true;
        }
        if self
            .equivalences
            .iter()
            .any(|cluster| cluster.contains(&sub) && cluster.contains(&sup))
        {
            return true;
        }
        ReachabilityIndex::new(self).is_subsumed(sub, sup)
    }

    /// Build a reachability index for repeated subsumption queries.
    #[must_use]
    pub fn reachability(&self) -> ReachabilityIndex {
        ReachabilityIndex::new(self)
    }

    /// Equivalence cluster containing `class`, if any.
    #[must_use]
    pub fn equivalent_classes(&self, class: EntityId) -> Option<&[EntityId]> {
        self.equivalences
            .iter()
            .find(|cluster| cluster.contains(&class))
            .map(Vec::as_slice)
    }

    /// Remove transitively redundant subsumption edges `(A,C)` when `(A,B)` and `(B,C)` exist.
    ///
    /// Produces a **minimal** cover of the same subsumption relation (HermiT-style direct edges).
    pub fn reduce_transitive_redundancy(&mut self) {
        let edges: Vec<(EntityId, EntityId)> = self.subsumptions.clone();
        if edges.is_empty() {
            return;
        }
        let graph = SubsumptionGraph::from_edges(&edges);
        let mut reduced = Vec::with_capacity(edges.len());
        for (sub, sup) in edges {
            let redundant = graph.direct_supers.get(&sub).is_some_and(|supers| {
                supers
                    .iter()
                    .any(|mid| *mid != sup && graph.reachable(*mid, sup))
            });
            if !redundant {
                reduced.push((sub, sup));
            }
        }
        reduced.sort_by_key(|(a, b)| (a.0, b.0));
        reduced.dedup();
        self.subsumptions = reduced;
        self.invalidate_graph_cache();
    }

    /// Collapse `#` vs `%23` duplicate entity ids (RDF/XML encoding artifact).
    pub fn canonicalize_entity_aliases(&mut self, ontology: &crate::Ontology) {
        let mut by_canon: HashMap<String, EntityId> = HashMap::new();
        for (id, record) in ontology.entities().iter() {
            let iri_str = ontology.resolve_iri(record.iri).unwrap_or("");
            let canon = iri_str.replace("%23", "#");
            by_canon.entry(canon).or_insert(id);
        }
        let remap = |id: EntityId| {
            ontology
                .entity(id)
                .ok()
                .and_then(|record| {
                    let iri_str = ontology.resolve_iri(record.iri).unwrap_or("");
                    let canon = iri_str.replace("%23", "#");
                    by_canon.get(&canon).copied()
                })
                .unwrap_or(id)
        };
        self.subsumptions = self
            .subsumptions
            .iter()
            .map(|&(sub, sup)| (remap(sub), remap(sup)))
            .collect();
        self.subsumptions.sort_by_key(|(a, b)| (a.0, b.0));
        self.subsumptions.dedup();
        self.invalidate_graph_cache();
        self.equivalences = self
            .equivalences
            .iter()
            .map(|cluster| {
                let mut mapped: Vec<EntityId> = cluster.iter().map(|&id| remap(id)).collect();
                mapped.sort_by_key(|id| id.0);
                mapped.dedup();
                mapped
            })
            .filter(|c| c.len() > 1)
            .collect();
        self.unsatisfiable = self
            .unsatisfiable
            .iter()
            .map(|&id| remap(id))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_subsumed_via_equivalence_cluster() {
        let a = EntityId(1);
        let b = EntityId(2);
        let tax = Taxonomy {
            equivalences: vec![vec![a, b]],
            ..Taxonomy::default()
        };
        assert!(tax.is_subsumed(a, b));
        assert!(tax.is_subsumed(b, a));
    }

    #[test]
    fn reduce_transitive_redundancy_removes_implied_edges() {
        let a = EntityId(1);
        let b = EntityId(2);
        let c = EntityId(3);
        let mut tax = Taxonomy {
            subsumptions: vec![(a, b), (b, c), (a, c)],
            ..Taxonomy::default()
        };
        tax.reduce_transitive_redundancy();
        assert_eq!(tax.subsumptions, vec![(a, b), (b, c)]);
    }

    #[test]
    fn reduce_transitive_redundancy_family_person_shortcuts() {
        let person = EntityId(1);
        let relative = EntityId(2);
        let aunt = EntityId(3);
        let man = EntityId(4);
        let brother = EntityId(5);
        let mut tax = Taxonomy {
            subsumptions: vec![
                (relative, person),
                (aunt, person),
                (aunt, relative),
                (man, person),
                (brother, person),
                (brother, man),
            ],
            ..Taxonomy::default()
        };
        tax.reduce_transitive_redundancy();
        assert!(!tax.subsumptions.contains(&(aunt, person)));
        assert!(!tax.subsumptions.contains(&(brother, person)));
        assert!(tax.subsumptions.contains(&(relative, person)));
        assert!(tax.subsumptions.contains(&(man, person)));
    }

    #[test]
    fn add_subsumption_and_contains_edge() {
        let a = EntityId(1);
        let b = EntityId(2);
        let mut tax = Taxonomy::default();
        assert!(tax.add_subsumption(a, b));
        assert!(!tax.add_subsumption(a, b));
        assert!(tax.contains_edge(a, b));
        assert!(tax.is_subsumed(a, b));
    }
}
