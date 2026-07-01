use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use crate::axiom::{Axiom, AxiomId};
use crate::entity::{EntityId, EntityRegistry};
use crate::error::{Error, Result};

/// Storage for ontology axioms.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AxiomStore {
    axioms: Vec<Axiom>,
    removed: HashSet<AxiomId>,
    inferred: HashSet<AxiomId>,
    dedup_index: HashMap<u64, AxiomId>,
}

fn axiom_fingerprint(axiom: &Axiom) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    axiom.hash(&mut hasher);
    hasher.finish()
}

impl AxiomStore {
    /// Create an empty axiom store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of stored axiom slots (including tombstoned).
    #[must_use]
    pub fn len(&self) -> usize {
        self.axioms.len()
    }

    /// Returns `true` if no axioms are stored.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.axioms.is_empty()
    }

    /// Look up an axiom by id.
    pub fn get(&self, id: AxiomId) -> Result<&Axiom> {
        if self.removed.contains(&id) {
            return Err(Error::InvalidAxiom(format!("removed AxiomId: {}", id.0)));
        }
        self.axioms
            .get(id.0 as usize)
            .ok_or_else(|| Error::InvalidAxiom(format!("unknown AxiomId: {}", id.0)))
    }

    /// Iterate over active axioms with their ids.
    pub fn iter(&self) -> impl Iterator<Item = (AxiomId, &Axiom)> {
        self.axioms.iter().enumerate().filter_map(|(i, axiom)| {
            let id = AxiomId(i as u32);
            if self.removed.contains(&id) {
                None
            } else {
                Some((id, axiom))
            }
        })
    }

    /// Iterate over active asserted (non-inferred) axioms.
    pub fn iter_asserted(&self) -> impl Iterator<Item = (AxiomId, &Axiom)> {
        self.iter().filter(|(id, _)| !self.inferred.contains(id))
    }

    /// Whether an axiom was added by reasoning merge (not user/asserted input).
    #[must_use]
    pub fn is_inferred(&self, id: AxiomId) -> bool {
        self.inferred.contains(&id)
    }

    /// Append a validated asserted axiom and return its id (idempotent on duplicates).
    pub fn push(&mut self, axiom: Axiom, registry: &EntityRegistry) -> Result<AxiomId> {
        self.push_with_provenance(axiom, registry, false)
    }

    /// Append a validated inferred axiom (from materialization merge).
    pub fn push_inferred(&mut self, axiom: Axiom, registry: &EntityRegistry) -> Result<AxiomId> {
        self.push_with_provenance(axiom, registry, true)
    }

    fn push_with_provenance(
        &mut self,
        axiom: Axiom,
        registry: &EntityRegistry,
        inferred: bool,
    ) -> Result<AxiomId> {
        let axiom = normalize_class_operands(axiom);
        axiom.validate(registry)?;
        let fp = axiom_fingerprint(&axiom);
        if let Some(&id) = self.dedup_index.get(&fp)
            && !self.removed.contains(&id)
            && self.axioms.get(id.0 as usize).is_some_and(|e| e == &axiom)
        {
            if !inferred {
                self.inferred.remove(&id);
            }
            return Ok(id);
        }
        if let Some((index, _)) = self.axioms.iter().enumerate().find(|(i, existing)| {
            !self.removed.contains(&AxiomId(*i as u32)) && **existing == axiom
        }) {
            let id = AxiomId(index as u32);
            self.dedup_index.insert(fp, id);
            if !inferred {
                self.inferred.remove(&id);
            }
            return Ok(id);
        }
        let id = AxiomId(
            u32::try_from(self.axioms.len())
                .map_err(|_| Error::InvalidAxiom("axiom store capacity exceeded".into()))?,
        );
        self.axioms.push(axiom);
        self.dedup_index.insert(fp, id);
        if inferred {
            self.inferred.insert(id);
        }
        Ok(id)
    }

    /// Tombstone all inferred axioms; returns ids removed.
    pub fn strip_inferred(&mut self) -> Vec<AxiomId> {
        let ids: Vec<AxiomId> = self
            .inferred
            .iter()
            .copied()
            .filter(|id| !self.removed.contains(id))
            .collect();
        for id in &ids {
            self.removed.insert(*id);
        }
        self.inferred.clear();
        ids
    }

    /// Look up an axiom by id including tombstoned entries.
    pub fn get_raw(&self, id: AxiomId) -> Result<&Axiom> {
        self.axioms
            .get(id.0 as usize)
            .ok_or_else(|| Error::InvalidAxiom(format!("unknown AxiomId: {}", id.0)))
    }

    /// Tombstone an axiom by id (ids remain stable; removed axioms are skipped in iteration).
    pub fn remove(&mut self, id: AxiomId) -> Result<()> {
        self.get(id)?;
        self.removed.insert(id);
        Ok(())
    }

    /// Whether an axiom id has been removed.
    #[must_use]
    pub fn is_removed(&self, id: AxiomId) -> bool {
        self.removed.contains(&id)
    }

    /// Count of non-removed axioms.
    #[must_use]
    pub fn active_len(&self) -> usize {
        self.axioms.len().saturating_sub(self.removed.len())
    }
}

fn normalize_class_operands(axiom: Axiom) -> Axiom {
    match axiom {
        Axiom::EquivalentClasses(mut classes) => {
            classes.sort_by_key(|id| id.0);
            Axiom::EquivalentClasses(classes)
        }
        Axiom::DisjointClasses(mut classes) => {
            classes.sort_by_key(|id| id.0);
            Axiom::DisjointClasses(classes)
        }
        Axiom::EquivalentObjectProperties(mut properties) => {
            properties.sort_by_key(|id| id.0);
            Axiom::EquivalentObjectProperties(properties)
        }
        Axiom::SameIndividual(mut individuals) => {
            individuals.sort_by_key(|id| id.0);
            Axiom::SameIndividual(individuals)
        }
        Axiom::DifferentIndividuals(mut individuals) => {
            individuals.sort_by_key(|id| id.0);
            Axiom::DifferentIndividuals(individuals)
        }
        Axiom::InverseObjectProperties { left, right } => {
            if left.0 <= right.0 {
                Axiom::InverseObjectProperties { left, right }
            } else {
                Axiom::InverseObjectProperties {
                    left: right,
                    right: left,
                }
            }
        }
        other => other,
    }
}

fn push_unique(vec: &mut Vec<EntityId>, value: EntityId) {
    match vec.binary_search(&value) {
        Ok(_) => {}
        Err(pos) => vec.insert(pos, value),
    }
}

/// Dense `EntityId` → sorted neighbor list (indexed by `EntityId.0`).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct DenseAdjacency {
    edges: Vec<Vec<EntityId>>,
}

impl DenseAdjacency {
    fn get(&self, id: EntityId) -> &[EntityId] {
        self.edges.get(id.0 as usize).map_or(&[], Vec::as_slice)
    }

    fn push_unique(&mut self, from: EntityId, to: EntityId) {
        let idx = from.0 as usize;
        if self.edges.len() <= idx {
            self.edges.resize(idx + 1, Vec::new());
        }
        push_unique(&mut self.edges[idx], to);
    }

    fn remove(&mut self, from: EntityId, to: EntityId) {
        if let Some(vec) = self.edges.get_mut(from.0 as usize)
            && let Ok(pos) = vec.binary_search(&to)
        {
            vec.remove(pos);
        }
    }
}

/// Union-find backed equivalence clusters (one representative map per relation).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct EquivalenceClusters {
    parent: HashMap<u32, u32>,
    clusters: HashMap<u32, HashSet<EntityId>>,
}

impl EquivalenceClusters {
    fn find(&self, id: EntityId) -> u32 {
        let mut root = id.0;
        while let Some(&p) = self.parent.get(&root) {
            if p == root {
                break;
            }
            root = p;
        }
        root
    }

    fn merge(&mut self, ids: &[EntityId]) {
        if ids.is_empty() {
            return;
        }
        let mut combined = HashSet::new();
        for &id in ids {
            combined.insert(id);
            let r = self.find(id);
            if let Some(existing) = self.clusters.get(&r) {
                combined.extend(existing.iter().copied());
            }
        }
        let root = combined
            .iter()
            .min_by_key(|id| id.0)
            .copied()
            .expect("non-empty cluster");
        for &id in &combined {
            self.parent.insert(id.0, root.0);
        }
        for key in combined.iter().map(|id| id.0).collect::<Vec<_>>() {
            if key != root.0 {
                self.clusters.remove(&key);
            }
        }
        self.clusters.insert(root.0, combined);
    }

    fn get(&self, class: EntityId) -> Option<&HashSet<EntityId>> {
        let root = self.find(class);
        self.clusters
            .get(&root)
            .filter(|set| set.len() > 1 && set.contains(&class))
    }
}

fn remove_pair_from_vec_map(
    map: &mut HashMap<EntityId, Vec<(EntityId, EntityId)>>,
    key: EntityId,
    value: (EntityId, EntityId),
) {
    if let Some(vec) = map.get_mut(&key) {
        vec.retain(|&v| v != value);
        if vec.is_empty() {
            map.remove(&key);
        }
    }
}

fn link_symmetric(map: &mut HashMap<EntityId, HashSet<EntityId>>, a: EntityId, b: EntityId) {
    map.entry(a).or_default().insert(b);
    map.entry(b).or_default().insert(a);
}

/// Secondary indexes over axioms for fast engine lookups.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AxiomIndex {
    subclass_of: DenseAdjacency,
    superclass_of: DenseAdjacency,
    subproperty_of: DenseAdjacency,
    superproperty_of: DenseAdjacency,
    property_domains: DenseAdjacency,
    property_ranges: DenseAdjacency,
    subclass_existentials: HashMap<EntityId, Vec<(EntityId, EntityId)>>,
    transitive_properties: HashSet<EntityId>,
    symmetric_properties: HashSet<EntityId>,
    reflexive_properties: HashSet<EntityId>,
    functional_properties: HashSet<EntityId>,
    inverse_functional_properties: HashSet<EntityId>,
    irreflexive_properties: HashSet<EntityId>,
    asymmetric_properties: HashSet<EntityId>,
    equivalent_classes: EquivalenceClusters,
    equivalent_properties: EquivalenceClusters,
    disjoint_with: HashMap<EntityId, HashSet<EntityId>>,
    inverse_of: HashMap<EntityId, EntityId>,
    classes_of: DenseAdjacency,
    individuals_of: DenseAdjacency,
    object_assertions_by_subject: HashMap<EntityId, Vec<(EntityId, EntityId)>>,
    object_assertions_by_object: HashMap<EntityId, Vec<(EntityId, EntityId)>>,
    same_as: EquivalenceClusters,
    different_from: HashMap<EntityId, HashSet<EntityId>>,
    by_kind: HashMap<&'static str, Vec<AxiomId>>,
}

impl AxiomIndex {
    /// Create empty indexes.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Rebuild all secondary indexes from active axioms.
    pub fn rebuild_from_store(&mut self, store: &AxiomStore) {
        *self = Self::default();
        for (id, axiom) in store.iter() {
            self.insert(id, axiom);
        }
    }

    /// Whether removing this axiom shape requires a full index rebuild.
    #[must_use]
    pub fn removal_needs_rebuild(axiom: &Axiom) -> bool {
        matches!(
            axiom,
            Axiom::EquivalentClasses(_)
                | Axiom::EquivalentObjectProperties(_)
                | Axiom::DisjointClasses(_)
                | Axiom::SameIndividual(_)
                | Axiom::DifferentIndividuals(_)
        )
    }

    /// Remove one axiom from secondary indexes (simple shapes only).
    pub fn remove(&mut self, id: AxiomId, axiom: &Axiom) {
        if let Some(ids) = self.by_kind.get_mut(axiom.kind_tag()) {
            ids.retain(|&aid| aid != id);
        }
        match axiom {
            Axiom::SubClassOf {
                subclass,
                superclass,
            } => {
                self.subclass_of.remove(*subclass, *superclass);
                self.superclass_of.remove(*superclass, *subclass);
            }
            Axiom::SubObjectPropertyOf {
                sub_property,
                super_property,
            } => {
                self.subproperty_of.remove(*sub_property, *super_property);
                self.superproperty_of.remove(*super_property, *sub_property);
            }
            Axiom::ObjectPropertyDomain { property, domain } => {
                self.property_domains.remove(*property, *domain);
            }
            Axiom::ObjectPropertyRange { property, range } => {
                self.property_ranges.remove(*property, *range);
            }
            Axiom::TransitiveObjectProperty(property) => {
                self.transitive_properties.remove(property);
            }
            Axiom::SubClassOfExistential {
                subclass,
                property,
                filler,
            } => {
                if let Some(entry) = self.subclass_existentials.get_mut(subclass) {
                    entry.retain(|&(p, f)| !(p == *property && f == *filler));
                }
            }
            Axiom::SymmetricObjectProperty(property) => {
                self.symmetric_properties.remove(property);
            }
            Axiom::ReflexiveObjectProperty(property) => {
                self.reflexive_properties.remove(property);
            }
            Axiom::FunctionalObjectProperty(property) => {
                self.functional_properties.remove(property);
            }
            Axiom::InverseFunctionalObjectProperty(property) => {
                self.inverse_functional_properties.remove(property);
            }
            Axiom::IrreflexiveObjectProperty(property) => {
                self.irreflexive_properties.remove(property);
            }
            Axiom::AsymmetricObjectProperty(property) => {
                self.asymmetric_properties.remove(property);
            }
            Axiom::InverseObjectProperties { left, right } => {
                self.inverse_of.remove(left);
                self.inverse_of.remove(right);
            }
            Axiom::ClassAssertion { individual, class } => {
                self.classes_of.remove(*individual, *class);
                self.individuals_of.remove(*class, *individual);
            }
            Axiom::ObjectPropertyAssertion {
                subject,
                property,
                object,
            } => {
                remove_pair_from_vec_map(
                    &mut self.object_assertions_by_subject,
                    *subject,
                    (*property, *object),
                );
                remove_pair_from_vec_map(
                    &mut self.object_assertions_by_object,
                    *object,
                    (*property, *subject),
                );
            }
            _ => {}
        }
    }

    /// Update indexes after inserting an axiom.
    pub fn insert(&mut self, id: AxiomId, axiom: &Axiom) {
        if !self
            .by_kind
            .entry(axiom.kind_tag())
            .or_default()
            .contains(&id)
        {
            self.by_kind.entry(axiom.kind_tag()).or_default().push(id);
        }

        match axiom {
            Axiom::SubClassOf {
                subclass,
                superclass,
            } => {
                self.subclass_of.push_unique(*subclass, *superclass);
                self.superclass_of.push_unique(*superclass, *subclass);
            }
            Axiom::SubObjectPropertyOf {
                sub_property,
                super_property,
            } => {
                self.subproperty_of
                    .push_unique(*sub_property, *super_property);
                self.superproperty_of
                    .push_unique(*super_property, *sub_property);
            }
            Axiom::ObjectPropertyDomain { property, domain } => {
                self.property_domains.push_unique(*property, *domain);
            }
            Axiom::ObjectPropertyRange { property, range } => {
                self.property_ranges.push_unique(*property, *range);
            }
            Axiom::TransitiveObjectProperty(property) => {
                self.transitive_properties.insert(*property);
            }
            Axiom::SubClassOfExistential {
                subclass,
                property,
                filler,
            } => {
                let entry = self.subclass_existentials.entry(*subclass).or_default();
                let pair = (*property, *filler);
                if !entry.contains(&pair) {
                    entry.push(pair);
                }
            }
            Axiom::SymmetricObjectProperty(property) => {
                self.symmetric_properties.insert(*property);
            }
            Axiom::ReflexiveObjectProperty(property) => {
                self.reflexive_properties.insert(*property);
            }
            Axiom::FunctionalObjectProperty(property) => {
                self.functional_properties.insert(*property);
            }
            Axiom::InverseFunctionalObjectProperty(property) => {
                self.inverse_functional_properties.insert(*property);
            }
            Axiom::IrreflexiveObjectProperty(property) => {
                self.irreflexive_properties.insert(*property);
            }
            Axiom::AsymmetricObjectProperty(property) => {
                self.asymmetric_properties.insert(*property);
            }
            Axiom::EquivalentClasses(classes) => {
                self.equivalent_classes.merge(classes);
            }
            Axiom::EquivalentObjectProperties(properties) => {
                self.equivalent_properties.merge(properties);
            }
            Axiom::DisjointClasses(classes) => {
                for i in 0..classes.len() {
                    for j in (i + 1)..classes.len() {
                        link_symmetric(&mut self.disjoint_with, classes[i], classes[j]);
                    }
                }
            }
            Axiom::InverseObjectProperties { left, right } => {
                self.inverse_of.insert(*left, *right);
                self.inverse_of.insert(*right, *left);
            }
            Axiom::ClassAssertion { individual, class } => {
                self.classes_of.push_unique(*individual, *class);
                self.individuals_of.push_unique(*class, *individual);
            }
            Axiom::ObjectPropertyAssertion {
                subject,
                property,
                object,
            } => {
                let pair = (*property, *object);
                let entry = self
                    .object_assertions_by_subject
                    .entry(*subject)
                    .or_default();
                if !entry.contains(&pair) {
                    entry.push(pair);
                }
                let reverse = (*property, *subject);
                let entry = self.object_assertions_by_object.entry(*object).or_default();
                if !entry.contains(&reverse) {
                    entry.push(reverse);
                }
            }
            Axiom::DataPropertyAssertion { .. }
            | Axiom::NegativeObjectPropertyAssertion { .. }
            | Axiom::NegativeDataPropertyAssertion { .. } => {}
            Axiom::SameIndividual(individuals) => {
                self.same_as.merge(individuals);
            }
            Axiom::DifferentIndividuals(individuals) => {
                for i in 0..individuals.len() {
                    for j in (i + 1)..individuals.len() {
                        link_symmetric(&mut self.different_from, individuals[i], individuals[j]);
                    }
                }
            }
        }
    }

    /// Direct superclasses declared for a class.
    #[must_use]
    pub fn direct_superclasses(&self, class: EntityId) -> &[EntityId] {
        self.subclass_of.get(class)
    }

    /// Direct subclasses declared for a class.
    #[must_use]
    pub fn direct_subclasses(&self, class: EntityId) -> &[EntityId] {
        self.superclass_of.get(class)
    }

    /// Existential restrictions declared for a subclass (`property`, `filler` pairs).
    #[must_use]
    pub fn existentials_of(&self, subclass: EntityId) -> &[(EntityId, EntityId)] {
        self.subclass_existentials
            .get(&subclass)
            .map_or(&[], Vec::as_slice)
    }

    /// Direct super-properties declared for a property.
    #[must_use]
    pub fn direct_superproperties(&self, property: EntityId) -> &[EntityId] {
        self.subproperty_of.get(property)
    }

    /// Direct sub-properties declared for a property.
    #[must_use]
    pub fn direct_subproperties(&self, property: EntityId) -> &[EntityId] {
        self.superproperty_of.get(property)
    }

    /// Domain classes declared for a property.
    #[must_use]
    pub fn domains_of(&self, property: EntityId) -> &[EntityId] {
        self.property_domains.get(property)
    }

    /// Range classes declared for a property.
    #[must_use]
    pub fn ranges_of(&self, property: EntityId) -> &[EntityId] {
        self.property_ranges.get(property)
    }

    /// Classes declared equivalent to the given class.
    #[must_use]
    pub fn equivalents_of(&self, class: EntityId) -> Option<&HashSet<EntityId>> {
        self.equivalent_classes.get(class)
    }

    /// Classes declared disjoint with the given class.
    #[must_use]
    pub fn disjoint_with(&self, class: EntityId) -> Option<&HashSet<EntityId>> {
        self.disjoint_with.get(&class)
    }

    /// Inverse property of the given object property, if declared.
    #[must_use]
    pub fn inverse_of(&self, property: EntityId) -> Option<EntityId> {
        self.inverse_of.get(&property).copied()
    }

    /// Properties declared transitive.
    #[must_use]
    pub fn transitive_properties(&self) -> &HashSet<EntityId> {
        &self.transitive_properties
    }

    /// Properties declared symmetric.
    #[must_use]
    pub fn symmetric_properties(&self) -> &HashSet<EntityId> {
        &self.symmetric_properties
    }

    /// Properties declared reflexive.
    #[must_use]
    pub fn reflexive_properties(&self) -> &HashSet<EntityId> {
        &self.reflexive_properties
    }

    /// Properties declared functional.
    #[must_use]
    pub fn functional_properties(&self) -> &HashSet<EntityId> {
        &self.functional_properties
    }

    /// Properties declared inverse-functional.
    #[must_use]
    pub fn inverse_functional_properties(&self) -> &HashSet<EntityId> {
        &self.inverse_functional_properties
    }

    /// Properties declared irreflexive.
    #[must_use]
    pub fn irreflexive_properties(&self) -> &HashSet<EntityId> {
        &self.irreflexive_properties
    }

    /// Properties declared asymmetric.
    #[must_use]
    pub fn asymmetric_properties(&self) -> &HashSet<EntityId> {
        &self.asymmetric_properties
    }

    /// Classes asserted for an individual.
    #[must_use]
    pub fn classes_of(&self, individual: EntityId) -> &[EntityId] {
        self.classes_of.get(individual)
    }

    /// Individuals asserted for a class.
    #[must_use]
    pub fn individuals_of(&self, class: EntityId) -> &[EntityId] {
        self.individuals_of.get(class)
    }

    /// Object property assertions with the given subject (`property`, `object` pairs).
    #[must_use]
    pub fn object_assertions_of(&self, subject: EntityId) -> &[(EntityId, EntityId)] {
        self.object_assertions_by_subject
            .get(&subject)
            .map_or(&[], Vec::as_slice)
    }

    /// Object property assertions with the given object (`property`, `subject` pairs).
    #[must_use]
    pub fn object_assertions_to(&self, object: EntityId) -> &[(EntityId, EntityId)] {
        self.object_assertions_by_object
            .get(&object)
            .map_or(&[], Vec::as_slice)
    }

    /// Properties declared equivalent to the given property.
    #[must_use]
    pub fn equivalent_properties_of(&self, property: EntityId) -> Option<&HashSet<EntityId>> {
        self.equivalent_properties.get(property)
    }

    /// Individuals declared same-as the given individual.
    #[must_use]
    pub fn same_as(&self, individual: EntityId) -> Option<&HashSet<EntityId>> {
        self.same_as.get(individual)
    }

    /// Individuals declared different-from the given individual.
    #[must_use]
    pub fn different_from(&self, individual: EntityId) -> Option<&HashSet<EntityId>> {
        self.different_from.get(&individual)
    }

    /// Axiom ids grouped by kind tag.
    #[must_use]
    pub fn by_kind(&self, kind: &str) -> &[AxiomId] {
        self.by_kind.get(kind).map_or(&[], Vec::as_slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{EntityKind, EntityRegistry};
    use crate::iri::InternPool;

    #[test]
    fn index_updates_on_insert() {
        let mut pool = InternPool::new();
        let mut registry = EntityRegistry::new();
        let a_iri = pool.intern("http://ex.org/A").expect("intern");
        let b_iri = pool.intern("http://ex.org/B").expect("intern");
        let a = registry
            .get_or_register(a_iri, "http://ex.org/A", EntityKind::Class)
            .expect("register");
        let b = registry
            .get_or_register(b_iri, "http://ex.org/B", EntityKind::Class)
            .expect("register");

        let mut store = AxiomStore::new();
        let mut index = AxiomIndex::new();
        let axiom = Axiom::SubClassOf {
            subclass: a,
            superclass: b,
        };
        let id = store.push(axiom, &registry).expect("push");
        index.insert(id, store.get(id).expect("get"));

        assert_eq!(index.direct_superclasses(a), &[b]);
        assert_eq!(index.direct_subclasses(b), &[a]);
        assert_eq!(index.by_kind("SubClassOf"), &[id]);
    }

    #[test]
    fn duplicate_axiom_deduped_in_store_and_index() {
        let mut pool = InternPool::new();
        let mut registry = EntityRegistry::new();
        let a_iri = pool.intern("http://ex.org/A").expect("intern");
        let b_iri = pool.intern("http://ex.org/B").expect("intern");
        let a = registry
            .get_or_register(a_iri, "http://ex.org/A", EntityKind::Class)
            .expect("register");
        let b = registry
            .get_or_register(b_iri, "http://ex.org/B", EntityKind::Class)
            .expect("register");

        let mut store = AxiomStore::new();
        let mut index = AxiomIndex::new();
        let axiom = Axiom::SubClassOf {
            subclass: a,
            superclass: b,
        };
        let id1 = store.push(axiom.clone(), &registry).expect("push");
        let id2 = store.push(axiom, &registry).expect("push");
        assert_eq!(id1, id2);
        assert_eq!(store.len(), 1);

        index.insert(id1, store.get(id1).expect("get"));
        index.insert(id2, store.get(id2).expect("get"));
        assert_eq!(index.direct_superclasses(a), &[b]);
    }

    #[test]
    fn subproperty_index_is_symmetric() {
        let mut pool = InternPool::new();
        let mut registry = EntityRegistry::new();
        let sub_iri = pool.intern("http://ex.org/sub").expect("intern");
        let sup_iri = pool.intern("http://ex.org/super").expect("intern");
        let sub = registry
            .get_or_register(sub_iri, "http://ex.org/sub", EntityKind::ObjectProperty)
            .expect("register");
        let sup = registry
            .get_or_register(sup_iri, "http://ex.org/super", EntityKind::ObjectProperty)
            .expect("register");

        let mut store = AxiomStore::new();
        let mut index = AxiomIndex::new();
        let axiom = Axiom::SubObjectPropertyOf {
            sub_property: sub,
            super_property: sup,
        };
        let id = store.push(axiom, &registry).expect("push");
        index.insert(id, store.get(id).expect("get"));

        assert_eq!(index.direct_superproperties(sub), &[sup]);
        assert_eq!(index.direct_subproperties(sup), &[sub]);
    }

    #[test]
    fn equivalence_closed_across_axioms() {
        let mut pool = InternPool::new();
        let mut registry = EntityRegistry::new();
        let mut ids = Vec::new();
        for label in ["A", "B", "C"] {
            let iri = pool
                .intern(&format!("http://ex.org/{label}"))
                .expect("intern");
            ids.push(
                registry
                    .get_or_register(iri, &format!("http://ex.org/{label}"), EntityKind::Class)
                    .expect("register"),
            );
        }
        let [a, b, c] = [ids[0], ids[1], ids[2]];

        let mut store = AxiomStore::new();
        let mut index = AxiomIndex::new();
        let id1 = store
            .push(Axiom::EquivalentClasses(vec![a, b]), &registry)
            .expect("push");
        index.insert(id1, store.get(id1).expect("get"));
        let id2 = store
            .push(Axiom::EquivalentClasses(vec![b, c]), &registry)
            .expect("push");
        index.insert(id2, store.get(id2).expect("get"));

        let equiv_a = index.equivalents_of(a).expect("equiv");
        assert!(equiv_a.contains(&b));
        assert!(equiv_a.contains(&c));
    }

    #[test]
    fn existential_indexed_separately_from_subclass() {
        let mut pool = InternPool::new();
        let mut registry = EntityRegistry::new();
        let c_iri = pool.intern("http://ex.org/C").expect("intern");
        let b_iri = pool.intern("http://ex.org/B").expect("intern");
        let p_iri = pool.intern("http://ex.org/p").expect("intern");
        let c = registry
            .get_or_register(c_iri, "http://ex.org/C", EntityKind::Class)
            .expect("register");
        let b = registry
            .get_or_register(b_iri, "http://ex.org/B", EntityKind::Class)
            .expect("register");
        let p = registry
            .get_or_register(p_iri, "http://ex.org/p", EntityKind::ObjectProperty)
            .expect("register");

        let mut store = AxiomStore::new();
        let mut index = AxiomIndex::new();
        let id = store
            .push(
                Axiom::SubClassOfExistential {
                    subclass: c,
                    property: p,
                    filler: b,
                },
                &registry,
            )
            .expect("push");
        index.insert(id, store.get(id).expect("get"));

        assert!(index.direct_superclasses(c).is_empty());
        assert!(index.ranges_of(p).is_empty());
        assert_eq!(index.existentials_of(c), &[(p, b)]);
    }

    #[test]
    fn equivalent_classes_operand_order_deduped() {
        let mut pool = InternPool::new();
        let mut registry = EntityRegistry::new();
        let a_iri = pool.intern("http://ex.org/A").expect("intern");
        let b_iri = pool.intern("http://ex.org/B").expect("intern");
        let a = registry
            .get_or_register(a_iri, "http://ex.org/A", EntityKind::Class)
            .expect("register");
        let b = registry
            .get_or_register(b_iri, "http://ex.org/B", EntityKind::Class)
            .expect("register");

        let mut store = AxiomStore::new();
        let id1 = store
            .push(Axiom::EquivalentClasses(vec![a, b]), &registry)
            .expect("push");
        let id2 = store
            .push(Axiom::EquivalentClasses(vec![b, a]), &registry)
            .expect("push");
        assert_eq!(id1, id2);
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn inverse_object_properties_operand_order_deduped() {
        let mut pool = InternPool::new();
        let mut registry = EntityRegistry::new();
        let p_iri = pool.intern("http://ex.org/p").expect("intern");
        let q_iri = pool.intern("http://ex.org/q").expect("intern");
        let p = registry
            .get_or_register(p_iri, "http://ex.org/p", EntityKind::ObjectProperty)
            .expect("register");
        let q = registry
            .get_or_register(q_iri, "http://ex.org/q", EntityKind::ObjectProperty)
            .expect("register");

        let mut store = AxiomStore::new();
        let id1 = store
            .push(
                Axiom::InverseObjectProperties { left: p, right: q },
                &registry,
            )
            .expect("push");
        let id2 = store
            .push(
                Axiom::InverseObjectProperties { left: q, right: p },
                &registry,
            )
            .expect("push");
        assert_eq!(id1, id2);
        assert_eq!(store.len(), 1);
    }
}
