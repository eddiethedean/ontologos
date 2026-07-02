//! Object-property taxonomy via concept surrogates (HermiT `classifyObjectProperties`).

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use ontologos_core::{Axiom, ClassExpr, DlAxiom, EntityId, EntityKind, Ontology, RoleExpr, Taxonomy};

use crate::Error;
use crate::dl_ontology::DlOntology;
use crate::tableau::{
    TableauSeed, infer_named_subsumptions_for, named_class_entails, role_equivalent_in_hierarchy,
    role_hierarchy_branch,
};

const FRESH_CLASS: &str = "urn:ontologos:internal:fresh-concept";
const FRESH_INDIVIDUAL: &str = "urn:ontologos:internal:fresh-individual";
const SURROGATE_NS: &str = "urn:ontologos:internal:role-surrogate:";
/// Full pairwise surrogate entailment is feasible below this role-expression count.
const FULL_SURROGATE_PAIRWISE_LIMIT: usize = 96;

fn query_object_property_entity(role: &RoleExpr) -> Option<EntityId> {
    match role {
        RoleExpr::Atomic(id) | RoleExpr::Inverse(id) => Some(*id),
    }
}

/// RBox `SubObjectPropertyOf` edges from asserted axioms (HermiT also surfaces these in role queries).
fn structural_sub_property_graph(ontology: &Ontology) -> HashMap<EntityId, HashSet<EntityId>> {
    let mut supers_of: HashMap<EntityId, HashSet<EntityId>> = HashMap::new();
    let mut note = |sub: EntityId, sup: EntityId| {
        supers_of.entry(sub).or_default().insert(sup);
    };
    for (_, axiom) in ontology.axioms().iter() {
        if let Axiom::SubObjectPropertyOf {
            sub_property,
            super_property,
        } = axiom
        {
            note(*sub_property, *super_property);
        }
    }
    for axiom in ontology.dl().axioms() {
        if let DlAxiom::SubObjectPropertyOf { sub, sup } = axiom {
            if let (Some(sub), Some(sup)) = (
                object_property_entity(ontology, sub),
                object_property_entity(ontology, sup),
            ) {
                note(sub, sup);
            }
        }
    }
    supers_of
}

fn object_property_entity(ontology: &Ontology, role: &RoleExpr) -> Option<EntityId> {
    match role {
        RoleExpr::Atomic(id) => ontology
            .entity(*id)
            .ok()
            .filter(|r| r.kind == EntityKind::ObjectProperty)
            .map(|_| *id),
        RoleExpr::Inverse(id) => object_property_entity(ontology, &RoleExpr::Atomic(*id)),
    }
}

/// Asserted subproperties of `query`, as atomic + inverse role expressions (OWL API shape).
fn structural_sub_role_exprs(
    ontology: &Ontology,
    query: EntityId,
    direct: bool,
) -> HashSet<RoleExpr> {
    let supers_of = structural_sub_property_graph(ontology);
    let mut children: HashMap<EntityId, HashSet<EntityId>> = HashMap::new();
    for (&sub, sups) in &supers_of {
        for &sup in sups {
            children.entry(sup).or_default().insert(sub);
        }
    }
    let mut props = HashSet::new();
    if direct {
        if let Some(kids) = children.get(&query) {
            props.extend(kids);
        }
    } else {
        let mut stack = vec![query];
        while let Some(sup) = stack.pop() {
            let Some(kids) = children.get(&sup) else {
                continue;
            };
            for &kid in kids {
                if props.insert(kid) {
                    stack.push(kid);
                }
            }
        }
    }
    let mut out = HashSet::new();
    for prop in props {
        out.insert(RoleExpr::Atomic(prop));
        out.insert(RoleExpr::Inverse(prop));
    }
    if !direct {
        let symmetric = ontology.axioms().iter().any(|(_, axiom)| {
            matches!(
                axiom,
                Axiom::SymmetricObjectProperty(id) if *id == query
            )
        });
        if symmetric {
            out.insert(RoleExpr::Inverse(query));
        }
    }
    out
}

fn build_surrogate_taxonomy(
    dl: &DlOntology,
    surrogates: &[EntityId],
    seed: &TableauSeed,
) -> Result<Taxonomy, Error> {
    let raw = infer_named_subsumptions_for(dl, surrogates, seed)?;
    let mut equiv = UnionFind::new(surrogates);
    for &(sub, sup) in &raw {
        if raw.iter().any(|&(a, b)| a == sup && b == sub) {
            equiv.union(sub, sup);
        }
    }
    let mut subsumptions: Vec<(EntityId, EntityId)> = raw
        .into_iter()
        .filter(|&(sub, sup)| !equiv.same_class(sub, sup))
        .collect();
    subsumptions.sort_by_key(|(a, b)| (a.0, b.0));
    subsumptions.dedup();
    let mut taxonomy = Taxonomy::from_parts(subsumptions, equiv.classes(surrogates), Vec::new());
    taxonomy.reduce_transitive_redundancy();
    Ok(taxonomy)
}

struct UnionFind {
    parent: HashMap<EntityId, EntityId>,
}

impl UnionFind {
    fn new(nodes: &[EntityId]) -> Self {
        let parent = nodes.iter().map(|&id| (id, id)).collect();
        Self { parent }
    }

    fn find(&mut self, id: EntityId) -> EntityId {
        let parent = self.parent.get(&id).copied().unwrap_or(id);
        if parent != id {
            let root = self.find(parent);
            self.parent.insert(id, root);
            root
        } else {
            id
        }
    }

    fn union(&mut self, a: EntityId, b: EntityId) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra != rb {
            self.parent.insert(ra, rb);
        }
    }

    fn same_class(&mut self, a: EntityId, b: EntityId) -> bool {
        self.find(a) == self.find(b)
    }

    fn classes(&mut self, nodes: &[EntityId]) -> Vec<Vec<EntityId>> {
        let mut buckets: HashMap<EntityId, Vec<EntityId>> = HashMap::new();
        for &id in nodes {
            buckets.entry(self.find(id)).or_default().push(id);
        }
        buckets.into_values().filter(|c| c.len() > 1).collect()
    }
}

struct RoleSurrogateContext {
    dl: DlOntology,
    role_to_surrogate: HashMap<RoleExpr, EntityId>,
    taxonomy: Option<Taxonomy>,
    seed: TableauSeed,
    role_equiv_classes: Vec<HashSet<RoleExpr>>,
    query_cache: RefCell<HashMap<EntityId, QuerySubCache>>,
    entails_cache: RefCell<HashMap<(EntityId, EntityId), bool>>,
}

#[derive(Clone)]
struct QuerySubCache {
    all: HashSet<RoleExpr>,
    direct: HashSet<RoleExpr>,
}

fn build_role_equivalence_classes(
    roles: &[RoleExpr],
    dl: &DlOntology,
    seed: &TableauSeed,
) -> Vec<HashSet<RoleExpr>> {
    if roles.is_empty() {
        return Vec::new();
    }
    let branch = role_hierarchy_branch(dl, seed);
    let mut parent: Vec<usize> = (0..roles.len()).collect();
    let find = |i: usize, parent: &mut [usize]| -> usize {
        let mut i = i;
        while parent[i] != i {
            let p = parent[i];
            parent[i] = parent[p];
            i = p;
        }
        i
    };
    for i in 0..roles.len() {
        for j in (i + 1)..roles.len() {
            if role_equivalent_in_hierarchy(&branch, &roles[i], &roles[j]) {
                let ri = find(i, &mut parent);
                let rj = find(j, &mut parent);
                if ri != rj {
                    parent[ri] = rj;
                }
            }
        }
    }
    let mut buckets: HashMap<usize, HashSet<RoleExpr>> = HashMap::new();
    for (idx, role) in roles.iter().enumerate() {
        buckets
            .entry(find(idx, &mut parent))
            .or_default()
            .insert(role.clone());
    }
    buckets.into_values().filter(|c| c.len() > 1).collect()
}

fn collect_relevant_role_expressions(ontology: &Ontology) -> HashSet<RoleExpr> {
    let mut roles = HashSet::new();
    for (id, record) in ontology.entities().iter() {
        if record.kind == EntityKind::ObjectProperty {
            roles.insert(RoleExpr::Atomic(id));
            roles.insert(RoleExpr::Inverse(id));
        }
    }
    for axiom in ontology.dl().axioms() {
        match axiom {
            DlAxiom::SubObjectPropertyOf { sub, sup } => {
                roles.insert(sub.clone());
                roles.insert(sup.clone());
            }
            DlAxiom::SubObjectPropertyChain {
                chain,
                super_property,
            } => {
                roles.extend(chain.iter().cloned());
                roles.insert(super_property.clone());
            }
            _ => {}
        }
    }
    roles
}

fn role_surrogate_iri(ontology: &Ontology, role: &RoleExpr) -> Result<String, Error> {
    let label = match role {
        RoleExpr::Atomic(id) => ontology.resolve_iri(ontology.entity(*id)?.iri)?.to_string(),
        RoleExpr::Inverse(id) => {
            let iri = ontology.resolve_iri(ontology.entity(*id)?.iri)?;
            format!("inv#{iri}")
        }
    };
    Ok(format!("{SURROGATE_NS}{label}"))
}

fn build_role_classification_ontology(
    base: &Ontology,
) -> Result<(Ontology, HashMap<RoleExpr, EntityId>), Error> {
    let mut ontology = base.clone();
    let fresh_class = ontology
        .entity_id(FRESH_CLASS, EntityKind::Class)
        .map_err(Error::Core)?;
    let fresh_individual = ontology
        .entity_id(FRESH_INDIVIDUAL, EntityKind::Individual)
        .map_err(Error::Core)?;
    let fresh_ce = ontology.dl_mut().intern_ce(ClassExpr::Atomic(fresh_class));
    ontology.dl_mut().push_axiom(DlAxiom::ClassAssertion {
        individual: fresh_individual,
        class: fresh_ce,
    });

    let mut role_to_surrogate = HashMap::new();
    for role in collect_relevant_role_expressions(&ontology) {
        let surrogate_iri = role_surrogate_iri(&ontology, &role)?;
        let surrogate = ontology
            .entity_id(&surrogate_iri, EntityKind::Class)
            .map_err(Error::Core)?;
        let surrogate_ce = ontology.dl_mut().intern_ce(ClassExpr::Atomic(surrogate));
        let exists_ce = ontology.dl_mut().intern_ce(ClassExpr::Some {
            property: role.clone(),
            filler: fresh_ce,
        });
        ontology
            .dl_mut()
            .push_axiom(DlAxiom::EquivalentClasses(vec![surrogate_ce, exists_ce]));
        role_to_surrogate.insert(role, surrogate);
    }
    Ok((ontology, role_to_surrogate))
}

#[doc(hidden)]
pub fn augment_for_role_classification(
    base: &Ontology,
) -> Result<(Ontology, HashMap<RoleExpr, EntityId>), Error> {
    build_role_classification_ontology(base)
}

impl RoleSurrogateContext {
    fn build(ontology: &Ontology, seed: &TableauSeed) -> Result<Self, Error> {
        let (augmented, role_to_surrogate) = build_role_classification_ontology(ontology)?;
        let dl = DlOntology::from_ontology(&augmented)?;
        Self::from_augmented(dl, role_to_surrogate, seed)
    }

    fn from_augmented(
        dl: DlOntology,
        role_to_surrogate: HashMap<RoleExpr, EntityId>,
        seed: &TableauSeed,
    ) -> Result<Self, Error> {
        let taxonomy = if role_to_surrogate.len() <= FULL_SURROGATE_PAIRWISE_LIMIT {
            let surrogates: Vec<EntityId> = role_to_surrogate.values().copied().collect();
            Some(build_surrogate_taxonomy(&dl, &surrogates, seed)?)
        } else {
            None
        };
        let role_keys: Vec<RoleExpr> = role_to_surrogate.keys().cloned().collect();
        let role_equiv_classes = build_role_equivalence_classes(&role_keys, &dl, seed);
        Ok(Self {
            dl,
            role_to_surrogate,
            taxonomy,
            seed: seed.clone(),
            role_equiv_classes,
            query_cache: RefCell::new(HashMap::new()),
            entails_cache: RefCell::new(HashMap::new()),
        })
    }

    fn entails_named(&self, sub: EntityId, sup: EntityId) -> Result<bool, Error> {
        if sub == sup {
            return Ok(true);
        }
        if let Some(&cached) = self.entails_cache.borrow().get(&(sub, sup)) {
            return Ok(cached);
        }
        let result = named_class_entails(&self.dl, sub, sup, &self.seed)?;
        self.entails_cache.borrow_mut().insert((sub, sup), result);
        Ok(result)
    }

    fn surrogates_equivalent(&self, left: EntityId, right: EntityId) -> Result<bool, Error> {
        if left == right {
            return Ok(true);
        }
        if let Some(taxonomy) = &self.taxonomy {
            if taxonomy
                .equivalences
                .iter()
                .any(|cluster| cluster.contains(&left) && cluster.contains(&right))
            {
                return Ok(true);
            }
            return Ok(taxonomy.is_subsumed(left, right) && taxonomy.is_subsumed(right, left));
        }
        Ok(self.entails_named(left, right)? && self.entails_named(right, left)?)
    }

    fn query_equiv_surrogates(
        &self,
        query_surrogate: EntityId,
    ) -> Result<HashSet<EntityId>, Error> {
        let mut out = HashSet::new();
        for &candidate in self.role_to_surrogate.values() {
            if self.surrogates_equivalent(query_surrogate, candidate)? {
                out.insert(candidate);
            }
        }
        Ok(out)
    }

    fn equivalent_roles(&self, property: &RoleExpr) -> Result<HashSet<RoleExpr>, Error> {
        let Some(query_surrogate) = self.query_surrogate(property) else {
            return Ok(HashSet::from([property.clone()]));
        };
        let query_equiv = self.query_equiv_surrogates(query_surrogate)?;
        Ok(self
            .role_to_surrogate
            .iter()
            .filter(|(_, surr)| query_equiv.contains(surr))
            .map(|(role, _)| role.clone())
            .collect())
    }

    fn subsumed_by_query(
        &self,
        candidate_surrogate: EntityId,
        query_equiv: &HashSet<EntityId>,
    ) -> Result<bool, Error> {
        if let Some(taxonomy) = &self.taxonomy {
            return Ok(query_equiv
                .iter()
                .any(|sup| taxonomy.is_subsumed(candidate_surrogate, *sup)));
        }
        for &sup in query_equiv {
            if self.entails_named(candidate_surrogate, sup)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn mid_subsumed_by_query(
        &self,
        mid: EntityId,
        query_equiv: &HashSet<EntityId>,
    ) -> Result<bool, Error> {
        if let Some(taxonomy) = &self.taxonomy {
            return Ok(query_equiv
                .iter()
                .any(|&sup| taxonomy.is_subsumed(mid, sup)));
        }
        for &sup in query_equiv {
            if self.entails_named(mid, sup)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn has_strict_intermediate(
        &self,
        candidate_surrogate: EntityId,
        query_equiv: &HashSet<EntityId>,
        sub_surrogates: &HashSet<EntityId>,
    ) -> Result<bool, Error> {
        if let Some(taxonomy) = &self.taxonomy {
            for &mid in sub_surrogates {
                if mid == candidate_surrogate || query_equiv.contains(&mid) {
                    continue;
                }
                if query_equiv.iter().any(|&sup| {
                    taxonomy.is_subsumed(candidate_surrogate, mid)
                        && taxonomy.is_subsumed(mid, sup)
                        && !taxonomy.is_subsumed(mid, candidate_surrogate)
                }) {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
        for &mid in sub_surrogates {
            if mid == candidate_surrogate || query_equiv.contains(&mid) {
                continue;
            }
            if self.entails_named(candidate_surrogate, mid)?
                && self.mid_subsumed_by_query(mid, query_equiv)?
                && !self.surrogates_equivalent(candidate_surrogate, mid)?
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn query_surrogate(&self, property: &RoleExpr) -> Option<EntityId> {
        self.role_to_surrogate.get(property).copied()
    }

    fn direct_sub_roles_with_taxonomy(
        &self,
        taxonomy: &Taxonomy,
        query_equiv: &HashSet<EntityId>,
    ) -> HashSet<RoleExpr> {
        let mut out = HashSet::new();
        for &query_sup in query_equiv {
            for sub in taxonomy.direct_subclasses(query_sup) {
                if query_equiv.contains(&sub) {
                    continue;
                }
                for (role, surr) in &self.role_to_surrogate {
                    if *surr == sub {
                        out.insert(role.clone());
                    }
                }
            }
        }
        out
    }

    fn expand_equivalent_role_expressions(&self, roles: HashSet<RoleExpr>) -> HashSet<RoleExpr> {
        if roles.is_empty() {
            return roles;
        }
        let mut out = roles;
        for class in &self.role_equiv_classes {
            if class.iter().any(|role| out.contains(role)) {
                out.extend(class.iter().cloned());
            }
        }
        out
    }

    fn is_strict_super_role(
        &self,
        query_surrogate: EntityId,
        candidate_surrogate: EntityId,
        query_equiv: &HashSet<EntityId>,
    ) -> Result<bool, Error> {
        if query_equiv.contains(&candidate_surrogate) {
            return Ok(false);
        }
        self.entails_named(query_surrogate, candidate_surrogate)
    }

    fn filter_strict_super_roles(
        &self,
        query_surrogate: EntityId,
        query_equiv: &HashSet<EntityId>,
        roles: HashSet<RoleExpr>,
    ) -> Result<HashSet<RoleExpr>, Error> {
        Ok(roles
            .into_iter()
            .filter(|role| {
                let Some(candidate_surrogate) = self.role_to_surrogate.get(role) else {
                    return true;
                };
                self.is_strict_super_role(query_surrogate, *candidate_surrogate, query_equiv)
                    .map(|is_super| !is_super)
                    .unwrap_or(true)
            })
            .collect())
    }

    fn compute_sub_roles_for_query(
        &self,
        query_surrogate: EntityId,
    ) -> Result<QuerySubCache, Error> {
        let query_equiv = self.query_equiv_surrogates(query_surrogate)?;
        let mut sub_surrogates = HashSet::new();
        let mut out = HashSet::new();
        for (role, candidate_surrogate) in &self.role_to_surrogate {
            if query_equiv.contains(candidate_surrogate) {
                continue;
            }
            if !self.subsumed_by_query(*candidate_surrogate, &query_equiv)? {
                continue;
            }
            sub_surrogates.insert(*candidate_surrogate);
            out.insert(role.clone());
        }
        let direct_base = if let Some(taxonomy) = &self.taxonomy {
            self.direct_sub_roles_with_taxonomy(taxonomy, &query_equiv)
        } else {
            out.iter()
                .filter(|role| {
                    let Some(candidate_surrogate) = self.role_to_surrogate.get(*role) else {
                        return false;
                    };
                    self.has_strict_intermediate(
                        *candidate_surrogate,
                        &query_equiv,
                        &sub_surrogates,
                    )
                    .map(|has| !has)
                    .unwrap_or(false)
                })
                .cloned()
                .collect()
        };
        let out = self.filter_strict_super_roles(query_surrogate, &query_equiv, out)?;
        let direct_base =
            self.filter_strict_super_roles(query_surrogate, &query_equiv, direct_base)?;
        let all = self.filter_strict_super_roles(
            query_surrogate,
            &query_equiv,
            self.expand_equivalent_role_expressions(out),
        )?;
        let direct = self.filter_strict_super_roles(
            query_surrogate,
            &query_equiv,
            self.expand_equivalent_role_expressions(direct_base),
        )?;
        Ok(QuerySubCache { all, direct })
    }

    fn sub_roles(&self, property: &RoleExpr, direct: bool) -> Result<HashSet<RoleExpr>, Error> {
        let Some(query_surrogate) = self.query_surrogate(property) else {
            return Ok(HashSet::new());
        };
        let mut cache = self.query_cache.borrow_mut();
        if let std::collections::hash_map::Entry::Vacant(e) = cache.entry(query_surrogate) {
            let computed = self.compute_sub_roles_for_query(query_surrogate)?;
            e.insert(computed);
        }
        let entry = cache.get(&query_surrogate).expect("query cache populated");
        let mut out = if direct {
            entry.direct.clone()
        } else {
            entry.all.clone()
        };
        if let Some(query_entity) = query_object_property_entity(property) {
            out.extend(structural_sub_role_exprs(self.dl.core(), query_entity, direct));
        }
        Ok(out)
    }
}

/// Classify object-property expressions into equivalence classes (HermiT `m_objectRoleHierarchy` nodes).
pub fn classify_object_property_expressions(
    ontology: &Ontology,
) -> Result<Vec<HashSet<RoleExpr>>, Error> {
    classify_object_property_expressions_with_seed(ontology, &TableauSeed::default())
}

/// Classify object-property expressions with a saturation-derived tableau seed.
pub fn classify_object_property_expressions_with_seed(
    ontology: &Ontology,
    seed: &TableauSeed,
) -> Result<Vec<HashSet<RoleExpr>>, Error> {
    let ctx = RoleSurrogateContext::build(ontology, seed)?;
    let mut classes = Vec::new();
    let mut assigned = HashSet::new();
    for role in ctx.role_to_surrogate.keys() {
        if !assigned.insert(role.clone()) {
            continue;
        }
        let class = ctx.equivalent_roles(role)?;
        assigned.extend(class.iter().cloned());
        classes.push(class);
    }
    Ok(classes)
}

/// OWL API `getEquivalentObjectProperties` via surrogate classification.
pub fn equivalent_object_property_expressions(
    ontology: &Ontology,
    property: &RoleExpr,
) -> Result<HashSet<RoleExpr>, Error> {
    equivalent_object_property_expressions_with_seed(ontology, property, &TableauSeed::default())
}

/// OWL API `getEquivalentObjectProperties` with a saturation-derived tableau seed.
pub fn equivalent_object_property_expressions_with_seed(
    ontology: &Ontology,
    property: &RoleExpr,
    seed: &TableauSeed,
) -> Result<HashSet<RoleExpr>, Error> {
    RoleSurrogateContext::build(ontology, seed)?.equivalent_roles(property)
}

/// OWL API `getSubObjectProperties` via surrogate classification (HermiT `m_objectRoleHierarchy`).
pub fn sub_object_property_expressions(
    ontology: &Ontology,
    property: &RoleExpr,
    direct: bool,
) -> Result<HashSet<RoleExpr>, Error> {
    sub_object_property_expressions_with_seed(ontology, property, direct, &TableauSeed::default())
}

/// OWL API `getSubObjectProperties` with a saturation-derived tableau seed.
pub fn sub_object_property_expressions_with_seed(
    ontology: &Ontology,
    property: &RoleExpr,
    direct: bool,
    seed: &TableauSeed,
) -> Result<HashSet<RoleExpr>, Error> {
    RoleSurrogateContext::build(ontology, seed)?.sub_roles(property, direct)
}

/// OWL API `getInverseObjectProperties` (HermiT: `getEquivalentObjectProperties(inverse(pe))`).
pub fn inverse_object_property_expressions(
    ontology: &Ontology,
    property: &RoleExpr,
) -> Result<HashSet<RoleExpr>, Error> {
    inverse_object_property_expressions_with_seed(ontology, property, &TableauSeed::default())
}

/// OWL API `getInverseObjectProperties` with a saturation-derived tableau seed.
pub fn inverse_object_property_expressions_with_seed(
    ontology: &Ontology,
    property: &RoleExpr,
    seed: &TableauSeed,
) -> Result<HashSet<RoleExpr>, Error> {
    equivalent_object_property_expressions_with_seed(ontology, &inverse_role(property), seed)
}

fn inverse_role(role: &RoleExpr) -> RoleExpr {
    match role {
        RoleExpr::Atomic(id) => RoleExpr::Inverse(*id),
        RoleExpr::Inverse(id) => RoleExpr::Atomic(*id),
    }
}

/// Prepared surrogate context for repeated object-property queries.
#[doc(hidden)]
pub struct PreparedRoleSurrogateContext(RoleSurrogateContext);

impl PreparedRoleSurrogateContext {
    #[doc(hidden)]
    pub fn from_augmented(
        dl: DlOntology,
        role_to_surrogate: HashMap<RoleExpr, EntityId>,
        seed: &TableauSeed,
    ) -> Result<Self, Error> {
        RoleSurrogateContext::from_augmented(dl, role_to_surrogate, seed).map(Self)
    }

    #[doc(hidden)]
    pub fn sub_object_property_expressions(
        &self,
        property: &RoleExpr,
        direct: bool,
    ) -> Result<HashSet<RoleExpr>, Error> {
        self.0.sub_roles(property, direct)
    }

    #[doc(hidden)]
    pub fn equivalent_object_property_expressions(
        &self,
        property: &RoleExpr,
    ) -> Result<HashSet<RoleExpr>, Error> {
        self.0.equivalent_roles(property)
    }
}

#[doc(hidden)]
pub fn sub_object_property_on_augmented(
    dl: DlOntology,
    role_to_surrogate: HashMap<RoleExpr, EntityId>,
    property: &RoleExpr,
    direct: bool,
    seed: &TableauSeed,
) -> Result<HashSet<RoleExpr>, Error> {
    PreparedRoleSurrogateContext::from_augmented(dl, role_to_surrogate, seed)?
        .sub_object_property_expressions(property, direct)
}

#[doc(hidden)]
pub fn equivalent_object_property_on_augmented(
    dl: DlOntology,
    role_to_surrogate: HashMap<RoleExpr, EntityId>,
    property: &RoleExpr,
    seed: &TableauSeed,
) -> Result<HashSet<RoleExpr>, Error> {
    PreparedRoleSurrogateContext::from_augmented(dl, role_to_surrogate, seed)?
        .equivalent_object_property_expressions(property)
}

#[doc(hidden)]
pub fn classify_object_property_on_augmented(
    dl: DlOntology,
    role_to_surrogate: HashMap<RoleExpr, EntityId>,
    seed: &TableauSeed,
) -> Result<Vec<HashSet<RoleExpr>>, Error> {
    let ctx = RoleSurrogateContext::from_augmented(dl, role_to_surrogate, seed)?;
    let mut classes = Vec::new();
    let mut assigned = HashSet::new();
    for role in ctx.role_to_surrogate.keys() {
        if !assigned.insert(role.clone()) {
            continue;
        }
        let class = ctx.equivalent_roles(role)?;
        assigned.extend(class.iter().cloned());
        classes.push(class);
    }
    Ok(classes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ontologos_parser::load_ontology;
    use std::path::PathBuf;

    #[test]
    fn hermit_inverse_cycle_equivalence_classes() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/axioms/hermit_reasoner_owlreasonertest_testgetinverseobjectpropertyexpressions.ofn",
        );
        let ontology = load_ontology(&path).expect("load inverse OFN");
        const NS: &str = "file:/c/test.owl#";
        let r = RoleExpr::Atomic(ontology.lookup_entity(&format!("{NS}r")).expect("r"));
        let s = RoleExpr::Atomic(ontology.lookup_entity(&format!("{NS}s")).expect("s"));
        let t = RoleExpr::Atomic(ontology.lookup_entity(&format!("{NS}t")).expect("t"));
        let inv_r = RoleExpr::Inverse(ontology.lookup_entity(&format!("{NS}r")).expect("r"));
        let inv_s = RoleExpr::Inverse(ontology.lookup_entity(&format!("{NS}s")).expect("s"));
        let inv_t = RoleExpr::Inverse(ontology.lookup_entity(&format!("{NS}t")).expect("t"));

        let r_inverses = inverse_object_property_expressions(&ontology, &r).expect("inverses of r");
        assert_eq!(
            r_inverses,
            HashSet::from([inv_r.clone(), s.clone(), inv_t.clone()])
        );

        let inv_r_inverses =
            inverse_object_property_expressions(&ontology, &inv_r).expect("inverses of inv(r)");
        assert_eq!(inv_r_inverses, HashSet::from([inv_s, r, t]));
    }

    #[test]
    #[ignore = "diagnostic: Bob knows subproperty counts"]
    fn bob_knows_subproperty_diag() {
        use crate::DlOntology;
        use crate::TableauSeed;
        use crate::augment_for_role_classification;
        use crate::tableau::named_class_entails;

        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit/reasoner/res/OWLLink/agent.owl");
        let ontology = load_ontology(&path).expect("load agent.owl");
        const NS: &str = "http://www.iyouit.eu/agent.owl#";
        let knows = RoleExpr::Atomic(
            ontology
                .lookup_entity(&format!("{NS}knows"))
                .expect("knows"),
        );
        let all = sub_object_property_expressions(&ontology, &knows, false).expect("all");
        let direct = sub_object_property_expressions(&ontology, &knows, true).expect("direct");
        eprintln!("direct={} all={}", direct.len(), all.len());
        let relation = RoleExpr::Atomic(
            ontology
                .lookup_entity(&format!("{NS}relation"))
                .expect("relation"),
        );
        eprintln!("relation in all: {}", all.contains(&relation));

        let (augmented, role_map) = augment_for_role_classification(&ontology).expect("augment");
        let dl = DlOntology::from_ontology(&augmented).expect("dl");
        let seed = TableauSeed::default();
        let knows_s = role_map.get(&knows).copied().expect("knows s");
        let relation_s = role_map.get(&relation).copied().expect("relation s");
        eprintln!(
            "knows ⊑ relation: {}",
            named_class_entails(&dl, knows_s, relation_s, &seed).expect("entails")
        );
        eprintln!(
            "relation ⊑ knows: {}",
            named_class_entails(&dl, relation_s, knows_s, &seed).expect("entails")
        );
    }
}
