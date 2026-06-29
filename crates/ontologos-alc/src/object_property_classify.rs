//! Object-property taxonomy via concept surrogates (HermiT `classifyObjectProperties`).

use std::collections::{HashMap, HashSet};

use ontologos_core::{
    ClassExpr, DlAxiom, EntityId, EntityKind, Ontology, RoleExpr, Taxonomy,
};

use crate::dl_ontology::DlOntology;
use crate::tableau::{infer_named_subsumptions_for, named_class_entails, TableauSeed};
use crate::Error;

const FRESH_CLASS: &str = "urn:ontologos:internal:fresh-concept";
const FRESH_INDIVIDUAL: &str = "urn:ontologos:internal:fresh-individual";
const SURROGATE_NS: &str = "urn:ontologos:internal:role-surrogate:";
/// Full pairwise surrogate entailment is feasible below this role-expression count.
const FULL_SURROGATE_PAIRWISE_LIMIT: usize = 96;

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
    let mut taxonomy = Taxonomy {
        subsumptions,
        equivalences: equiv.classes(surrogates),
        unsatisfiable: Vec::new(),
    };
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
        buckets
            .into_values()
            .filter(|c| c.len() > 1)
            .collect()
    }
}

struct RoleSurrogateContext {
    dl: DlOntology,
    role_to_surrogate: HashMap<RoleExpr, EntityId>,
    taxonomy: Option<Taxonomy>,
    seed: TableauSeed,
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
    let fresh_ce = ontology
        .dl_mut()
        .intern_ce(ClassExpr::Atomic(fresh_class));
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
        let surrogate_ce = ontology
            .dl_mut()
            .intern_ce(ClassExpr::Atomic(surrogate));
        let exists_ce = ontology.dl_mut().intern_ce(ClassExpr::Some {
            property: role.clone(),
            filler: fresh_ce,
        });
        ontology.dl_mut().push_axiom(DlAxiom::EquivalentClasses(vec![
            surrogate_ce,
            exists_ce,
        ]));
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
        Ok(Self {
            dl,
            role_to_surrogate,
            taxonomy,
            seed: seed.clone(),
        })
    }

    fn query_surrogate(&self, property: &RoleExpr) -> Option<EntityId> {
        self.role_to_surrogate.get(property).copied()
    }

    fn entails_named(&self, sub: EntityId, sup: EntityId) -> Result<bool, Error> {
        named_class_entails(&self.dl, sub, sup, &self.seed)
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
            return Ok(
                taxonomy.is_subsumed(left, right) && taxonomy.is_subsumed(right, left),
            );
        }
        Ok(self.entails_named(left, right)? && self.entails_named(right, left)?)
    }

    fn query_equiv_surrogates(&self, query_surrogate: EntityId) -> Result<HashSet<EntityId>, Error> {
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

    fn direct_sub_roles(
        &self,
        query_equiv: &HashSet<EntityId>,
    ) -> HashSet<RoleExpr> {
        let Some(taxonomy) = &self.taxonomy else {
            return HashSet::new();
        };
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

    fn sub_roles(&self, property: &RoleExpr, direct: bool) -> Result<HashSet<RoleExpr>, Error> {
        let Some(query_surrogate) = self.query_surrogate(property) else {
            return Ok(HashSet::new());
        };
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
        if !direct {
            return Ok(out);
        }
        if self.taxonomy.is_some() {
            return Ok(self.direct_sub_roles(&query_equiv));
        }
        Ok(out
            .into_iter()
            .filter(|role| {
                let Some(candidate_surrogate) = self.role_to_surrogate.get(role) else {
                    return false;
                };
                self.has_strict_intermediate(*candidate_surrogate, &query_equiv, &sub_surrogates)
                    .map(|has| !has)
                    .unwrap_or(false)
            })
            .collect())
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

#[doc(hidden)]
pub fn sub_object_property_on_augmented(
    dl: DlOntology,
    role_to_surrogate: HashMap<RoleExpr, EntityId>,
    property: &RoleExpr,
    direct: bool,
    seed: &TableauSeed,
) -> Result<HashSet<RoleExpr>, Error> {
    RoleSurrogateContext::from_augmented(dl, role_to_surrogate, seed)?.sub_roles(property, direct)
}

#[doc(hidden)]
pub fn equivalent_object_property_on_augmented(
    dl: DlOntology,
    role_to_surrogate: HashMap<RoleExpr, EntityId>,
    property: &RoleExpr,
    seed: &TableauSeed,
) -> Result<HashSet<RoleExpr>, Error> {
    RoleSurrogateContext::from_augmented(dl, role_to_surrogate, seed)?.equivalent_roles(property)
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
        assert_eq!(r_inverses, HashSet::from([inv_r.clone(), s.clone(), inv_t.clone()]));

        let inv_r_inverses =
            inverse_object_property_expressions(&ontology, &inv_r).expect("inverses of inv(r)");
        assert_eq!(inv_r_inverses, HashSet::from([inv_s, r, t]));
    }
}
