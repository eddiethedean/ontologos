use std::collections::{HashMap, HashSet};

use crate::axiom::{Axiom, AxiomId};
use crate::entity::{EntityId, EntityRegistry};
use crate::error::{Error, Result};

/// Storage for ontology axioms.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AxiomStore {
    axioms: Vec<Axiom>,
}

impl AxiomStore {
    /// Create an empty axiom store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of stored axioms.
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
        self.axioms
            .get(id.0 as usize)
            .ok_or_else(|| Error::InvalidAxiom(format!("unknown AxiomId: {}", id.0)))
    }

    /// Iterate over all axioms with their ids.
    pub fn iter(&self) -> impl Iterator<Item = (AxiomId, &Axiom)> {
        self.axioms
            .iter()
            .enumerate()
            .map(|(i, axiom)| (AxiomId(i as u32), axiom))
    }

    /// Append a validated axiom and return its id.
    pub fn push(&mut self, axiom: Axiom, registry: &EntityRegistry) -> Result<AxiomId> {
        axiom.validate(registry)?;
        let id = AxiomId(
            u32::try_from(self.axioms.len())
                .map_err(|_| Error::InvalidAxiom("axiom store capacity exceeded".into()))?,
        );
        self.axioms.push(axiom);
        Ok(id)
    }
}

/// Secondary indexes over axioms for fast engine lookups.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AxiomIndex {
    subclass_of: HashMap<EntityId, Vec<EntityId>>,
    superclass_of: HashMap<EntityId, Vec<EntityId>>,
    subproperty_of: HashMap<EntityId, Vec<EntityId>>,
    property_domains: HashMap<EntityId, Vec<EntityId>>,
    property_ranges: HashMap<EntityId, Vec<EntityId>>,
    transitive_properties: HashSet<EntityId>,
    by_kind: HashMap<&'static str, Vec<AxiomId>>,
}

impl AxiomIndex {
    /// Create empty indexes.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Update indexes after inserting an axiom.
    pub fn insert(&mut self, id: AxiomId, axiom: &Axiom) {
        self.by_kind.entry(axiom.kind_tag()).or_default().push(id);

        match axiom {
            Axiom::SubClassOf {
                subclass,
                superclass,
            } => {
                self.subclass_of
                    .entry(*subclass)
                    .or_default()
                    .push(*superclass);
                self.superclass_of
                    .entry(*superclass)
                    .or_default()
                    .push(*subclass);
            }
            Axiom::SubObjectPropertyOf {
                sub_property,
                super_property,
            } => {
                self.subproperty_of
                    .entry(*sub_property)
                    .or_default()
                    .push(*super_property);
            }
            Axiom::ObjectPropertyDomain { property, domain } => {
                self.property_domains
                    .entry(*property)
                    .or_default()
                    .push(*domain);
            }
            Axiom::ObjectPropertyRange { property, range } => {
                self.property_ranges
                    .entry(*property)
                    .or_default()
                    .push(*range);
            }
            Axiom::TransitiveObjectProperty(property) => {
                self.transitive_properties.insert(*property);
            }
            Axiom::EquivalentClasses(_)
            | Axiom::DisjointClasses(_)
            | Axiom::InverseObjectProperties { .. } => {}
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

    /// Direct super-properties declared for a property.
    #[must_use]
    pub fn direct_superproperties(&self, property: EntityId) -> &[EntityId] {
        self.subproperty_of
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

    /// Properties declared transitive.
    #[must_use]
    pub fn transitive_properties(&self) -> &HashSet<EntityId> {
        &self.transitive_properties
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
            .get_or_register(a_iri, EntityKind::Class)
            .expect("register");
        let b = registry
            .get_or_register(b_iri, EntityKind::Class)
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
}
