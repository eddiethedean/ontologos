//! OWL 2 property hierarchy regularity and simplicity checks (HermiT parity).

use std::collections::{HashMap, HashSet};

use ontologos_core::{Axiom, DlAxiom, EntityId, Ontology, RoleExpr};

use crate::Result;

/// Returns whether the object-property axioms form a regular hierarchy.
pub fn is_property_hierarchy_regular(ontology: &Ontology) -> Result<bool> {
    let store = ontology.dl();
    let mut subprops: HashSet<(EntityId, EntityId)> = HashSet::new();
    let mut chains: Vec<(Vec<RoleExpr>, RoleExpr)> = Vec::new();
    let mut inverses: HashMap<EntityId, EntityId> = HashMap::new();
    let mut reflexive: HashSet<EntityId> = HashSet::new();
    let mut transitive: HashSet<EntityId> = HashSet::new();

    for axiom in store.axioms() {
        match axiom {
            DlAxiom::SubObjectPropertyOf { sub, sup } => {
                if let (RoleExpr::Atomic(a), RoleExpr::Atomic(b)) = (sub, sup) {
                    subprops.insert((*a, *b));
                }
            }
            DlAxiom::SubObjectPropertyChain {
                chain,
                super_property,
            } => {
                chains.push((chain.clone(), super_property.clone()));
            }
            _ => {}
        }
    }
    for (_, axiom) in ontology.axioms().iter() {
        match axiom {
            Axiom::InverseObjectProperties { left, right } => {
                inverses.insert(*left, *right);
                inverses.insert(*right, *left);
            }
            Axiom::TransitiveObjectProperty(prop) => {
                transitive.insert(*prop);
            }
            Axiom::ReflexiveObjectProperty(prop) => {
                reflexive.insert(*prop);
            }
            _ => {}
        }
    }
    let equiv = build_role_equivalences(ontology);
    for axiom in store.axioms() {
        if let DlAxiom::TransitiveObjectProperty(RoleExpr::Atomic(prop)) = axiom {
            transitive.insert(*prop);
        }
    }

    let subprops = extend_subprops_from_chains(&subprops, &chains, &inverses, &equiv);
    let closure = saturate_subproperties(&subprops);
    if chains.iter().any(|(chain, _)| chain.len() >= 2)
        && subproperty_cycle_intersects_chain(&subprops, &chains, &equiv)
    {
        return Ok(false);
    }
    for (chain, sup) in &chains {
        if !chain_regularity_ok(
            chain,
            sup,
            &chains,
            &closure,
            &inverses,
            &reflexive,
            &transitive,
            &equiv,
        ) {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Returns whether every object property used in cardinality restrictions is simple.
pub fn is_property_hierarchy_simple(ontology: &Ontology) -> Result<bool> {
    let non_simple = compute_non_simple_roles(ontology)?;
    let store = ontology.dl();
    for axiom in store.axioms() {
        let props = cardinality_roles_in_axiom(store, axiom);
        if props.iter().any(|p| non_simple.contains(p)) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn build_role_equivalences(ontology: &Ontology) -> HashMap<EntityId, EntityId> {
    let mut parent: HashMap<EntityId, EntityId> = HashMap::new();
    fn find(parent: &mut HashMap<EntityId, EntityId>, x: EntityId) -> EntityId {
        let p = parent.get(&x).copied().unwrap_or(x);
        if p != x {
            let root = find(parent, p);
            parent.insert(x, root);
            root
        } else {
            x
        }
    }
    fn unite(parent: &mut HashMap<EntityId, EntityId>, a: EntityId, b: EntityId) {
        let ra = find(parent, a);
        let rb = find(parent, b);
        if ra != rb {
            parent.insert(ra, rb);
        }
    }
    for (_, axiom) in ontology.axioms().iter() {
        if let Axiom::EquivalentObjectProperties(props) = axiom {
            for pair in props.windows(2) {
                unite(&mut parent, pair[0], pair[1]);
            }
            if let (Some(&first), Some(&last)) = (props.first(), props.last()) {
                unite(&mut parent, first, last);
            }
        }
    }
    parent
}

fn roles_equivalent(
    a: &RoleExpr,
    b: &RoleExpr,
    inverses: &HashMap<EntityId, EntityId>,
    equiv: &HashMap<EntityId, EntityId>,
) -> bool {
    if roles_equal(a, b, inverses) {
        return true;
    }
    let atomic = |r: &RoleExpr| -> Option<EntityId> {
        match r {
            RoleExpr::Atomic(id) => Some(*id),
            RoleExpr::Inverse(id) => inverses.get(id).copied(),
        }
    };
    match (atomic(a), atomic(b)) {
        (Some(x), Some(y)) => {
            let mut parent = equiv.clone();
            find_equiv(&mut parent, x) == find_equiv(&mut parent, y)
        }
        _ => false,
    }
}

fn find_equiv(parent: &mut HashMap<EntityId, EntityId>, x: EntityId) -> EntityId {
    let p = parent.get(&x).copied().unwrap_or(x);
    if p != x {
        let root = find_equiv(parent, p);
        parent.insert(x, root);
        root
    } else {
        x
    }
}

/// Derive atomic subproperty edges used by RIA cycle detection (chain endpoints).
fn extend_subprops_from_chains(
    subprops: &HashSet<(EntityId, EntityId)>,
    chains: &[(Vec<RoleExpr>, RoleExpr)],
    inverses: &HashMap<EntityId, EntityId>,
    equiv: &HashMap<EntityId, EntityId>,
) -> HashSet<(EntityId, EntityId)> {
    let mut out = subprops.clone();
    for (chain, sup) in chains {
        let Some(first) = chain.first() else {
            continue;
        };
        if let (RoleExpr::Atomic(a), RoleExpr::Atomic(b)) = (first, sup) {
            out.insert((*a, *b));
        }
        if chain.len() >= 2 {
            if let Some(RoleExpr::Atomic(last)) = chain.last() {
                if let RoleExpr::Atomic(b) = sup {
                    if !roles_equivalent(&RoleExpr::Atomic(*last), sup, inverses, equiv) {
                        out.insert((*last, *b));
                    }
                }
            }
        }
    }
    out
}

fn chain_roles_share_equivalence_class(
    chain: &[RoleExpr],
    sup: &RoleExpr,
    equiv: &HashMap<EntityId, EntityId>,
) -> bool {
    let mut rep: Option<EntityId> = None;
    for role in chain {
        let RoleExpr::Atomic(id) = role else {
            return false;
        };
        let mut parent = equiv.clone();
        let r = find_equiv(&mut parent, *id);
        rep = Some(match rep {
            None => r,
            Some(prev) if prev == r => prev,
            _ => return false,
        });
    }
    let RoleExpr::Atomic(sup_id) = sup else {
        return false;
    };
    let mut parent = equiv.clone();
    rep == Some(find_equiv(&mut parent, *sup_id))
}

fn roles_are_inverse_adjacent(
    left: &RoleExpr,
    right: &RoleExpr,
    inverses: &HashMap<EntityId, EntityId>,
) -> bool {
    if let (RoleExpr::Atomic(a), RoleExpr::Atomic(b)) = (left, right) {
        if inverses.get(a) == Some(b) {
            return true;
        }
    }
    if let RoleExpr::Inverse(inner) = right {
        if let RoleExpr::Atomic(a) = left {
            return *inner == *a;
        }
    }
    if let RoleExpr::Inverse(inner) = left {
        if let RoleExpr::Atomic(b) = right {
            return *inner == *b;
        }
    }
    false
}

fn normalize_subprops_by_equiv(
    subprops: &HashSet<(EntityId, EntityId)>,
    equiv: &HashMap<EntityId, EntityId>,
) -> HashSet<(EntityId, EntityId)> {
    let mut out = HashSet::new();
    for (a, b) in subprops {
        let mut parent = equiv.clone();
        let ra = find_equiv(&mut parent, *a);
        let mut parent = equiv.clone();
        let rb = find_equiv(&mut parent, *b);
        if ra != rb {
            out.insert((ra, rb));
        }
    }
    out
}

fn subproperty_cycle_intersects_chain(
    subprops: &HashSet<(EntityId, EntityId)>,
    chains: &[(Vec<RoleExpr>, RoleExpr)],
    equiv: &HashMap<EntityId, EntityId>,
) -> bool {
    let subprops = normalize_subprops_by_equiv(subprops, equiv);
    let mut graph: HashMap<EntityId, Vec<EntityId>> = HashMap::new();
    for (a, b) in subprops {
        graph.entry(a).or_default().push(b);
    }
    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    let mut cyclic_nodes = HashSet::new();

    fn dfs(
        node: EntityId,
        graph: &HashMap<EntityId, Vec<EntityId>>,
        visiting: &mut HashSet<EntityId>,
        visited: &mut HashSet<EntityId>,
        cyclic: &mut HashSet<EntityId>,
    ) {
        if visiting.contains(&node) {
            cyclic.insert(node);
            return;
        }
        if visited.contains(&node) {
            return;
        }
        visiting.insert(node);
        if let Some(nexts) = graph.get(&node) {
            for &next in nexts {
                dfs(next, graph, visiting, visited, cyclic);
            }
        }
        visiting.remove(&node);
        visited.insert(node);
    }

    for &start in graph.keys() {
        dfs(
            start,
            &graph,
            &mut visiting,
            &mut visited,
            &mut cyclic_nodes,
        );
    }
    if cyclic_nodes.is_empty() {
        return false;
    }
    chains.iter().any(|(chain, _)| {
        chain.len() >= 2
            && chain.iter().any(|role| {
                if let RoleExpr::Atomic(id) = role {
                    cyclic_nodes.contains(id)
                } else {
                    false
                }
            })
    })
}

fn saturate_subproperties(
    subprops: &HashSet<(EntityId, EntityId)>,
) -> HashSet<(EntityId, EntityId)> {
    let mut closure = subprops.clone();
    let mut changed = true;
    while changed {
        changed = false;
        let pairs: Vec<_> = closure.iter().copied().collect();
        for (a, b) in &pairs {
            for (c, d) in &pairs {
                if *b == *c && closure.insert((*a, *d)) {
                    changed = true;
                }
            }
            if closure.insert((*a, *a)) {
                changed = true;
            }
        }
    }
    closure
}

fn role_inverse(role: &RoleExpr, inverses: &HashMap<EntityId, EntityId>) -> Option<RoleExpr> {
    match role {
        RoleExpr::Atomic(id) => inverses.get(id).map(|inv| RoleExpr::Atomic(*inv)),
        RoleExpr::Inverse(id) => Some(RoleExpr::Atomic(*id)),
    }
}

fn roles_equal(a: &RoleExpr, b: &RoleExpr, inverses: &HashMap<EntityId, EntityId>) -> bool {
    if a == b {
        return true;
    }
    if let Some(inv) = role_inverse(a, inverses) {
        if inv == *b {
            return true;
        }
    }
    if let Some(inv) = role_inverse(b, inverses) {
        if *a == inv {
            return true;
        }
    }
    false
}

#[allow(clippy::too_many_arguments)]
fn chain_regularity_ok(
    chain: &[RoleExpr],
    sup: &RoleExpr,
    all_chains: &[(Vec<RoleExpr>, RoleExpr)],
    closure: &HashSet<(EntityId, EntityId)>,
    inverses: &HashMap<EntityId, EntityId>,
    reflexive: &HashSet<EntityId>,
    transitive: &HashSet<EntityId>,
    equiv: &HashMap<EntityId, EntityId>,
) -> bool {
    if chain_roles_share_equivalence_class(chain, sup, equiv) {
        return true;
    }
    if chain.len() >= 3 {
        for i in 0..chain.len().saturating_sub(1) {
            if roles_are_inverse_adjacent(&chain[i], &chain[i + 1], inverses) {
                return false;
            }
        }
    }
    if chain.len() >= 2 && !chain_roles_share_equivalence_class(chain, sup, equiv) {
        let last = chain.len() - 1;
        for (i, role) in chain.iter().enumerate().take(last) {
            if roles_equivalent(role, sup, inverses, equiv) {
                return false;
            }
            if i + 1 < last && roles_equivalent(sup, &chain[i + 1], inverses, equiv) {
                return false;
            }
        }
    }
    if chain.len() == 2 {
        let right = &chain[1];
        if roles_equivalent(sup, right, inverses, equiv)
            && !chain_roles_share_equivalence_class(chain, sup, equiv)
        {
            return false;
        }
    }
    if chain.len() < 2 {
        return true;
    }
    for i in 0..chain.len().saturating_sub(1) {
        let left = &chain[i];
        let right = &chain[i + 1];
        let pair = [left.clone(), right.clone()];
        if pair_chain_in_closure(
            &pair, right, all_chains, closure, inverses, reflexive, transitive,
        ) {
            return false;
        }
    }
    true
}

fn pair_chain_in_closure(
    pair: &[RoleExpr],
    target: &RoleExpr,
    all_chains: &[(Vec<RoleExpr>, RoleExpr)],
    closure: &HashSet<(EntityId, EntityId)>,
    inverses: &HashMap<EntityId, EntityId>,
    reflexive: &HashSet<EntityId>,
    transitive: &HashSet<EntityId>,
) -> bool {
    for (chain, sup) in all_chains {
        if chain.len() == 2
            && chain[0] == pair[0]
            && chain[1] == pair[1]
            && roles_equal(sup, target, inverses)
        {
            let exempt = pair_exempt_from_regularity(&pair[0], &pair[1], reflexive, transitive);
            if !exempt {
                return true;
            }
        }
    }
    if let (RoleExpr::Atomic(a), RoleExpr::Atomic(b)) = (&pair[0], &pair[1]) {
        if closure.contains(&(*a, *b)) && roles_equal(&RoleExpr::Atomic(*b), target, inverses) {
            return !pair_exempt_from_regularity(&pair[0], &pair[1], reflexive, transitive);
        }
    }
    false
}

fn pair_exempt_from_regularity(
    left: &RoleExpr,
    right: &RoleExpr,
    reflexive: &HashSet<EntityId>,
    transitive: &HashSet<EntityId>,
) -> bool {
    if let RoleExpr::Atomic(id) = left {
        if reflexive.contains(id) || transitive.contains(id) {
            if let RoleExpr::Atomic(rid) = right {
                return id == rid;
            }
        }
    }
    false
}

fn compute_non_simple_roles(ontology: &Ontology) -> Result<HashSet<EntityId>> {
    let store = ontology.dl();
    let mut non_simple: HashSet<EntityId> = HashSet::new();
    let mut subprops: HashSet<(EntityId, EntityId)> = HashSet::new();
    let mut chains: Vec<(Vec<RoleExpr>, RoleExpr)> = Vec::new();
    let mut transitive: HashSet<EntityId> = HashSet::new();
    let mut reflexive: HashSet<EntityId> = HashSet::new();
    let mut inverses: HashMap<EntityId, EntityId> = HashMap::new();

    for (_, axiom) in ontology.axioms().iter() {
        match axiom {
            Axiom::TransitiveObjectProperty(prop) => {
                transitive.insert(*prop);
                non_simple.insert(*prop);
            }
            Axiom::ReflexiveObjectProperty(prop) => {
                reflexive.insert(*prop);
                non_simple.insert(*prop);
            }
            Axiom::InverseObjectProperties { left, right } => {
                inverses.insert(*left, *right);
                inverses.insert(*right, *left);
            }
            _ => {}
        }
    }
    for axiom in store.axioms() {
        if let DlAxiom::TransitiveObjectProperty(RoleExpr::Atomic(prop)) = axiom {
            transitive.insert(*prop);
            non_simple.insert(*prop);
        }
    }

    for axiom in store.axioms() {
        match axiom {
            DlAxiom::SubObjectPropertyOf { sub, sup } => {
                if let (RoleExpr::Atomic(a), RoleExpr::Atomic(b)) = (sub, sup) {
                    subprops.insert((*a, *b));
                }
            }
            DlAxiom::SubObjectPropertyChain {
                chain,
                super_property,
            } => {
                chains.push((chain.clone(), super_property.clone()));
                for part in chain {
                    if let RoleExpr::Atomic(id) = part {
                        non_simple.insert(*id);
                    }
                }
                if let RoleExpr::Atomic(id) = super_property {
                    non_simple.insert(*id);
                }
            }
            DlAxiom::TransitiveObjectProperty(RoleExpr::Atomic(prop)) => {
                transitive.insert(*prop);
                non_simple.insert(*prop);
            }
            _ => {}
        }
    }
    for (_, axiom) in ontology.axioms().iter() {
        if let Axiom::SubObjectPropertyOf {
            sub_property: sub,
            super_property: sup,
        } = axiom
        {
            subprops.insert((*sub, *sup));
        }
    }

    let closure = saturate_subproperties(&subprops);
    let mut changed = true;
    while changed {
        changed = false;
        for (sub, sup) in &closure {
            if non_simple.contains(sub) && non_simple.insert(*sup) {
                changed = true;
            }
            if (transitive.contains(sup) || reflexive.contains(sup) || inverses.contains_key(sup))
                && non_simple.insert(*sub)
            {
                changed = true;
            }
        }
        for (left, right) in &inverses {
            if non_simple.contains(left) && non_simple.insert(*right) {
                changed = true;
            }
        }
        for (chain, _) in &chains {
            for part in chain {
                if let RoleExpr::Atomic(id) = part {
                    if non_simple.insert(*id) {
                        changed = true;
                    }
                }
            }
        }
    }
    let _ = store;
    Ok(non_simple)
}

fn cardinality_roles_in_axiom(store: &ontologos_core::DlStore, axiom: &DlAxiom) -> Vec<EntityId> {
    use ontologos_core::ClassExpr;
    let mut out = Vec::new();
    fn collect_ce(
        store: &ontologos_core::DlStore,
        ce: ontologos_core::CeId,
        out: &mut Vec<EntityId>,
    ) {
        let Some(expr) = store.ce(ce) else {
            return;
        };
        match expr {
            ClassExpr::MinCardinality { property, .. }
            | ClassExpr::MaxCardinality { property, .. }
            | ClassExpr::ExactCardinality { property, .. } => {
                if let RoleExpr::Atomic(id) = property {
                    out.push(*id);
                }
            }
            ClassExpr::And(children) | ClassExpr::Or(children) => {
                for child in children {
                    collect_ce(store, *child, out);
                }
            }
            _ => {}
        }
    }
    if let DlAxiom::SubClassOf { sup, .. } = axiom {
        collect_ce(store, *sup, &mut out);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ontologos_parser::load_ofn_from_str;

    fn load_axioms(axioms: &str) -> Ontology {
        let ofn = format!(
            "Prefix(:=<file:/c/test.owl#>)\n\
             Prefix(owl:=<http://www.w3.org/2002/07/owl#>)\n\
             Ontology(<file:/c/test.owl#>\n{axioms}\n)\n"
        );
        load_ofn_from_str(&ofn).expect("load")
    }

    #[test]
    fn ria_cycle_is_regular() {
        let ont = load_axioms(
            "SubObjectPropertyOf(:A :B) SubObjectPropertyOf(:B :C) \
             SubObjectPropertyOf(:C :D) SubObjectPropertyOf(:D :A)",
        );
        assert!(is_property_hierarchy_regular(&ont).unwrap());
    }

    #[test]
    fn ria5_equivalent_cycle_regular() {
        let ont = load_axioms(
            "SubObjectPropertyOf(ObjectPropertyChain(:R :Q :P) :P) \
             SubObjectPropertyOf(ObjectPropertyChain(:P :S) :L) \
             SubObjectPropertyOf(:L :R) SubObjectPropertyOf(:R :L)",
        );
        assert!(is_property_hierarchy_regular(&ont).unwrap());
    }

    #[test]
    fn ria_chain_inverse_irregular() {
        let ont = load_axioms(
            "SubObjectPropertyOf(ObjectPropertyChain(:R :Q) :P) InverseObjectProperties(:P :Q)",
        );
        assert!(!is_property_hierarchy_regular(&ont).unwrap());
    }

    #[test]
    fn simple_roles_transitive_subproperty() {
        let ont = load_axioms(
            "TransitiveObjectProperty(:R) SubObjectPropertyOf(:R :P) \
             SubClassOf(:C ObjectMinCardinality(2 :P))",
        );
        let non_simple = compute_non_simple_roles(&ont).unwrap();
        let p = ont
            .lookup_entity("http://file:/c/test.owl#P")
            .or_else(|| ont.lookup_entity("file:/c/test.owl#P"));
        assert!(p.is_some(), "missing P entity");
        assert!(
            non_simple.contains(&p.unwrap()),
            "non_simple={non_simple:?}"
        );
        assert!(!is_property_hierarchy_simple(&ont).unwrap());
    }
}
