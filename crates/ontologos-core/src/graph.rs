use std::collections::{HashMap, HashSet};

use crate::axiom::{Axiom, AxiomId};
use crate::entity::{EntityId, EntityRegistry};
use crate::error::{Error, Result};

/// Storage for ontology axioms.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AxiomStore {
    axioms: Vec<Axiom>,
    removed: HashSet<AxiomId>,
    inferred: HashSet<AxiomId>,
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
        if let Some((index, _)) = self.axioms.iter().enumerate().find(|(i, existing)| {
            !self.removed.contains(&AxiomId(*i as u32)) && **existing == axiom
        }) {
            let id = AxiomId(index as u32);
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
    if !vec.contains(&value) {
        vec.push(value);
    }
}

fn link_symmetric(map: &mut HashMap<EntityId, HashSet<EntityId>>, a: EntityId, b: EntityId) {
    map.entry(a).or_default().insert(b);
    map.entry(b).or_default().insert(a);
}

fn merge_equivalence_class(map: &mut HashMap<EntityId, HashSet<EntityId>>, classes: &[EntityId]) {
    let mut component: HashSet<EntityId> = classes.iter().copied().collect();
    for id in classes {
        if let Some(existing) = map.get(id) {
            component.extend(existing.iter().copied());
        }
    }
    let members: Vec<EntityId> = component.into_iter().collect();
    for i in 0..members.len() {
        for j in (i + 1)..members.len() {
            link_symmetric(map, members[i], members[j]);
        }
    }
}

/// Secondary indexes over axioms for fast engine lookups.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AxiomIndex {
    subclass_of: HashMap<EntityId, Vec<EntityId>>,
    superclass_of: HashMap<EntityId, Vec<EntityId>>,
    subproperty_of: HashMap<EntityId, Vec<EntityId>>,
    superproperty_of: HashMap<EntityId, Vec<EntityId>>,
    property_domains: HashMap<EntityId, Vec<EntityId>>,
    property_ranges: HashMap<EntityId, Vec<EntityId>>,
    subclass_existentials: HashMap<EntityId, Vec<(EntityId, EntityId)>>,
    transitive_properties: HashSet<EntityId>,
    symmetric_properties: HashSet<EntityId>,
    reflexive_properties: HashSet<EntityId>,
    functional_properties: HashSet<EntityId>,
    asymmetric_properties: HashSet<EntityId>,
    equivalent_classes: HashMap<EntityId, HashSet<EntityId>>,
    equivalent_properties: HashMap<EntityId, HashSet<EntityId>>,
    disjoint_with: HashMap<EntityId, HashSet<EntityId>>,
    inverse_of: HashMap<EntityId, EntityId>,
    classes_of: HashMap<EntityId, Vec<EntityId>>,
    individuals_of: HashMap<EntityId, Vec<EntityId>>,
    object_assertions_by_subject: HashMap<EntityId, Vec<(EntityId, EntityId)>>,
    object_assertions_by_object: HashMap<EntityId, Vec<(EntityId, EntityId)>>,
    same_as: HashMap<EntityId, HashSet<EntityId>>,
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
                push_unique(self.subclass_of.entry(*subclass).or_default(), *superclass);
                push_unique(
                    self.superclass_of.entry(*superclass).or_default(),
                    *subclass,
                );
            }
            Axiom::SubObjectPropertyOf {
                sub_property,
                super_property,
            } => {
                push_unique(
                    self.subproperty_of.entry(*sub_property).or_default(),
                    *super_property,
                );
                push_unique(
                    self.superproperty_of.entry(*super_property).or_default(),
                    *sub_property,
                );
            }
            Axiom::ObjectPropertyDomain { property, domain } => {
                push_unique(self.property_domains.entry(*property).or_default(), *domain);
            }
            Axiom::ObjectPropertyRange { property, range } => {
                push_unique(self.property_ranges.entry(*property).or_default(), *range);
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
            Axiom::AsymmetricObjectProperty(property) => {
                self.asymmetric_properties.insert(*property);
            }
            Axiom::EquivalentClasses(classes) => {
                merge_equivalence_class(&mut self.equivalent_classes, classes);
            }
            Axiom::EquivalentObjectProperties(properties) => {
                merge_equivalence_class(&mut self.equivalent_properties, properties);
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
                push_unique(self.classes_of.entry(*individual).or_default(), *class);
                push_unique(self.individuals_of.entry(*class).or_default(), *individual);
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
            Axiom::SameIndividual(individuals) => {
                merge_equivalence_class(&mut self.same_as, individuals);
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
        self.subclass_of.get(&class).map_or(&[], Vec::as_slice)
    }

    /// Direct subclasses declared for a class.
    #[must_use]
    pub fn direct_subclasses(&self, class: EntityId) -> &[EntityId] {
        self.superclass_of.get(&class).map_or(&[], Vec::as_slice)
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
        self.subproperty_of
            .get(&property)
            .map_or(&[], Vec::as_slice)
    }

    /// Direct sub-properties declared for a property.
    #[must_use]
    pub fn direct_subproperties(&self, property: EntityId) -> &[EntityId] {
        self.superproperty_of
            .get(&property)
            .map_or(&[], Vec::as_slice)
    }

    /// Domain classes declared for a property.
    #[must_use]
    pub fn domains_of(&self, property: EntityId) -> &[EntityId] {
        self.property_domains
            .get(&property)
            .map_or(&[], Vec::as_slice)
    }

    /// Range classes declared for a property.
    #[must_use]
    pub fn ranges_of(&self, property: EntityId) -> &[EntityId] {
        self.property_ranges
            .get(&property)
            .map_or(&[], Vec::as_slice)
    }

    /// Classes declared equivalent to the given class.
    #[must_use]
    pub fn equivalents_of(&self, class: EntityId) -> Option<&HashSet<EntityId>> {
        self.equivalent_classes.get(&class)
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

    /// Properties declared asymmetric.
    #[must_use]
    pub fn asymmetric_properties(&self) -> &HashSet<EntityId> {
        &self.asymmetric_properties
    }

    /// Classes asserted for an individual.
    #[must_use]
    pub fn classes_of(&self, individual: EntityId) -> &[EntityId] {
        self.classes_of.get(&individual).map_or(&[], Vec::as_slice)
    }

    /// Individuals asserted for a class.
    #[must_use]
    pub fn individuals_of(&self, class: EntityId) -> &[EntityId] {
        self.individuals_of.get(&class).map_or(&[], Vec::as_slice)
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
        self.equivalent_properties.get(&property)
    }

    /// Individuals declared same-as the given individual.
    #[must_use]
    pub fn same_as(&self, individual: EntityId) -> Option<&HashSet<EntityId>> {
        self.same_as.get(&individual)
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
