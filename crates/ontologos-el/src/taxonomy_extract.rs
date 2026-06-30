use std::collections::{HashMap, HashSet};

use ontologos_core::{EntityId, Ontology, Taxonomy};

use crate::graph::CompletionGraph;

/// Extract a reduced taxonomy from a saturated completion graph.
pub fn extract_taxonomy(ontology: &Ontology, graph: &CompletionGraph) -> Taxonomy {
    let subsumptions = graph.subsumptions();
    let mut uf = UnionFind::new();

    for &(sub, sup) in subsumptions {
        if graph.is_subsumed(sup, sub) {
            uf.union(sub, sup);
        }
    }
    let mut seen_equiv: HashSet<EntityId> = HashSet::new();
    for (class, _) in ontology.entities().iter() {
        if seen_equiv.contains(&class) {
            continue;
        }
        if let Some(cluster) = ontology.equivalents_of(class) {
            let members: Vec<EntityId> = std::iter::once(class)
                .chain(cluster.iter().copied())
                .collect();
            for i in 1..members.len() {
                uf.union(members[0], members[i]);
            }
            seen_equiv.extend(members.iter().copied());
        }
    }

    let mut equivalences: HashMap<EntityId, HashSet<EntityId>> = HashMap::new();
    for &(sub, sup) in subsumptions {
        if graph.is_subsumed(sup, sub) {
            let rep = uf.find(sub);
            equivalences.entry(rep).or_default().insert(sub);
            equivalences.entry(rep).or_default().insert(sup);
        }
    }
    seen_equiv.clear();
    for (class, _) in ontology.entities().iter() {
        if seen_equiv.contains(&class) {
            continue;
        }
        if let Some(cluster) = ontology.equivalents_of(class) {
            let rep = uf.find(class);
            equivalences
                .entry(rep)
                .or_default()
                .extend(std::iter::once(class).chain(cluster.iter().copied()));
            seen_equiv.extend(cluster.iter().copied());
            seen_equiv.insert(class);
        }
    }

    let equiv_vec: Vec<Vec<EntityId>> = equivalences
        .into_values()
        .filter(|c| c.len() > 1)
        .map(|mut c| {
            let mut v: Vec<_> = c.drain().collect();
            v.sort_by_key(|id| id.0);
            v
        })
        .collect();

    let bottom = find_bottom_class(ontology);
    let mut unsatisfiable = Vec::new();
    if let Some(bot) = bottom {
        for (class, record) in ontology.entities().iter() {
            if record.kind == ontologos_core::EntityKind::Class && graph.is_subsumed(class, bot) {
                unsatisfiable.push(class);
            }
        }
    }

    let mut direct_subsumptions = Vec::new();
    let classes: HashSet<EntityId> = ontology
        .entities()
        .iter()
        .filter(|(_, r)| r.kind == ontologos_core::EntityKind::Class)
        .map(|(id, _)| id)
        .collect();

    for &sub in &classes {
        let rep_sub = uf.find(sub);
        let supers: HashSet<EntityId> = subsumptions
            .iter()
            .filter_map(|&(s, sup)| (s == sub).then_some(sup))
            .filter(|&sup| uf.find(sup) != rep_sub)
            .collect();

        for &sup in &supers {
            let is_redundant = supers.iter().any(|&mid| {
                mid != sup
                    && mid != sub
                    && graph.is_subsumed(mid, sup)
                    && graph.is_subsumed(sub, mid)
            });
            if !is_redundant {
                direct_subsumptions.push((sub, sup));
            }
        }
    }

    direct_subsumptions.sort_by_key(|(a, b)| (a.0, b.0));

    Taxonomy::from_parts(direct_subsumptions, equiv_vec, unsatisfiable)
}

fn find_bottom_class(ontology: &Ontology) -> Option<EntityId> {
    const NOTHING_IRIS: &[&str] = &[
        "http://www.w3.org/2002/07/owl#Nothing",
        "http://www.w3.org/2002/07/owl#Bottom",
    ];
    for iri in NOTHING_IRIS {
        if let Some(id) = ontology.lookup_entity(iri) {
            return Some(id);
        }
    }
    None
}

struct UnionFind {
    parent: HashMap<u32, u32>,
}

impl UnionFind {
    fn new() -> Self {
        Self {
            parent: HashMap::new(),
        }
    }

    fn find(&mut self, id: EntityId) -> EntityId {
        let key = id.0;
        self.parent.entry(key).or_insert(key);
        let root = self.parent[&key];
        if root != key {
            let found = self.find(EntityId(root));
            self.parent.insert(key, found.0);
            found
        } else {
            id
        }
    }

    fn union(&mut self, a: EntityId, b: EntityId) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra != rb {
            self.parent.insert(ra.0, rb.0);
        }
    }
}
