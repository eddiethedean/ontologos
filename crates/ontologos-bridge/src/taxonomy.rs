use std::collections::{HashMap, HashSet};

use ontologos_core::EntityId;
use petgraph::algo::has_path_connecting;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;

/// Transitive reduction: keep edge `(a,b)` only when no alternate path `a ↝ b` exists.
pub fn reduce_subsumptions(pairs: &[(EntityId, EntityId)]) -> Vec<(EntityId, EntityId)> {
    if pairs.is_empty() {
        return Vec::new();
    }

    let mut index: HashMap<EntityId, NodeIndex> = HashMap::new();
    let mut graph = DiGraph::<EntityId, ()>::new();

    for &(sub, sup) in pairs {
        let sub_idx = *index.entry(sub).or_insert_with(|| graph.add_node(sub));
        let sup_idx = *index.entry(sup).or_insert_with(|| graph.add_node(sup));
        graph.add_edge(sub_idx, sup_idx, ());
    }

    let mut direct = Vec::new();
    for &(sub, sup) in pairs {
        let Some(&sub_idx) = index.get(&sub) else {
            continue;
        };
        let Some(&sup_idx) = index.get(&sup) else {
            continue;
        };

        let redundant = graph
            .edges(sub_idx)
            .filter(|edge| edge.target() != sup_idx)
            .any(|edge| has_path_connecting(&graph, edge.target(), sup_idx, None));

        if !redundant {
            direct.push((sub, sup));
        }
    }

    direct.sort_by_key(|(a, b)| (a.0, b.0));
    direct
}

/// Build equivalence clusters from mutual subsumption pairs.
pub fn equivalence_clusters(pairs: &[(EntityId, EntityId)]) -> Vec<Vec<EntityId>> {
    let mut subs: HashMap<EntityId, HashSet<EntityId>> = HashMap::new();
    for &(sub, sup) in pairs {
        subs.entry(sub).or_default().insert(sup);
    }

    let mut uf: HashMap<EntityId, EntityId> = HashMap::new();
    for &id in subs.keys() {
        uf.insert(id, id);
    }
    for id in subs.keys().copied().collect::<Vec<_>>() {
        if let Some(sups) = subs.get(&id) {
            for sup in sups {
                if subs.get(sup).is_some_and(|rev| rev.contains(&id)) {
                    union(&mut uf, id, *sup);
                }
            }
        }
    }

    let mut clusters: HashMap<EntityId, HashSet<EntityId>> = HashMap::new();
    for id in uf.keys().copied().collect::<Vec<_>>() {
        let root = find(&uf, id);
        clusters.entry(root).or_default().insert(id);
    }

    clusters
        .into_values()
        .filter(|c| c.len() > 1)
        .map(|mut c| {
            let mut v: Vec<_> = c.drain().collect();
            v.sort_by_key(|id| id.0);
            v
        })
        .collect()
}

fn find(uf: &HashMap<EntityId, EntityId>, mut x: EntityId) -> EntityId {
    let mut parent = x;
    while uf.get(&parent).copied().unwrap_or(parent) != parent {
        parent = uf[&parent];
    }
    while uf.get(&x).copied().unwrap_or(x) != parent {
        let next = uf[&x];
        x = next;
    }
    parent
}

fn union(uf: &mut HashMap<EntityId, EntityId>, a: EntityId, b: EntityId) {
    let ra = find(uf, a);
    let rb = find(uf, b);
    if ra != rb {
        uf.insert(ra, rb);
    }
}

#[cfg(test)]
mod tests {
    use ontologos_core::EntityId;

    use super::*;

    #[test]
    fn drops_transitive_hop() {
        let a = EntityId(1);
        let b = EntityId(2);
        let c = EntityId(3);
        let pairs = vec![(a, b), (b, c), (a, c)];
        let reduced = reduce_subsumptions(&pairs);
        assert!(!reduced.contains(&(a, c)));
        assert!(reduced.contains(&(a, b)));
        assert!(reduced.contains(&(b, c)));
    }
}
