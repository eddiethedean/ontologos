use std::path::Path;

use crate::axiom::{Axiom, AxiomId};
use crate::dl::DlStore;
use crate::dirty::{axiom_signature, DirtySet, OntologyRevision};
use crate::entity::{EntityId, EntityKind, EntityRecord, EntityRegistry};
use crate::error::{Error, Result};
use crate::graph::{AxiomIndex, AxiomStore};
use crate::iri::{validate_iri, InternPool, IriId};
use crate::parse_meta::ParseMeta;

/// In-memory ontology with interned IRIs, typed entities, and indexed axioms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ontology {
    pub(crate) iris: InternPool,
    pub(crate) entities: EntityRegistry,
    pub(crate) axioms: AxiomStore,
    pub(crate) index: AxiomIndex,
    pub(crate) revision: OntologyRevision,
    pub(crate) dirty: DirtySet,
    pub(crate) dl: DlStore,
    #[doc(hidden)]
    pub parse_meta: Option<ParseMeta>,
}

impl Default for Ontology {
    fn default() -> Self {
        Self::new()
    }
}

impl Ontology {
    /// Create an empty ontology.
    #[must_use]
    pub fn new() -> Self {
        Self {
            iris: InternPool::new(),
            entities: EntityRegistry::new(),
            axioms: AxiomStore::new(),
            index: AxiomIndex::new(),
            revision: OntologyRevision::default(),
            dirty: DirtySet::default(),
            dl: DlStore::new(),
            parse_meta: None,
        }
    }

    /// Create a builder for programmatic ontology construction.
    #[must_use]
    pub fn builder() -> OntologyBuilder {
        OntologyBuilder::new()
    }

    /// Load an ontology from a file path.
    ///
    /// Use [`ontologos_parser::load_ontology`] for OWL/RDF file loading.
    pub fn from_file(_path: impl AsRef<Path>) -> Result<Self> {
        Err(Error::ParseNotAvailable)
    }

    /// Parse metadata from the last file load (not present for JSON/builder ontologies).
    #[must_use]
    pub fn parse_meta(&self) -> Option<&ParseMeta> {
        self.parse_meta.as_ref()
    }

    /// Attach parse metadata (used by `ontologos-parser`).
    pub fn set_parse_meta(&mut self, meta: ParseMeta) {
        self.parse_meta = Some(meta);
    }

    /// Number of registered entities.
    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Number of stored axioms (active, excluding tombstoned).
    #[must_use]
    pub fn axiom_count(&self) -> usize {
        self.axioms.active_len()
    }

    /// DL class expressions and generalized axioms.
    #[must_use]
    pub fn dl(&self) -> &DlStore {
        &self.dl
    }

    /// Mutable DL store (parser / reasoner ingestion).
    pub fn dl_mut(&mut self) -> &mut DlStore {
        &mut self.dl
    }

    /// Monotonic edit revision (incremented on add/remove).
    #[must_use]
    pub fn revision(&self) -> OntologyRevision {
        self.revision
    }

    /// Pending axiom edits since the last [`Self::clear_dirty`].
    #[must_use]
    pub fn dirty(&self) -> &DirtySet {
        &self.dirty
    }

    /// Clear pending dirty flags after incremental engines consume edits.
    ///
    /// Prefer letting profile engines flush dirty state; clearing manually between edits
    /// without re-classifying can leave incremental sessions stale.
    pub fn clear_dirty(&mut self) {
        self.dirty.clear();
    }

    /// Remove all inferred axioms (from RL/RDFS materialization) and rebuild indexes.
    #[must_use]
    pub fn strip_inferred_axioms(&mut self) -> usize {
        let removed = self.axioms.strip_inferred();
        if !removed.is_empty() {
            self.index.rebuild_from_store(&self.axioms);
            self.revision.bump();
        }
        removed.len()
    }

    /// Add an inferred axiom from materialization (does not mark dirty).
    pub fn add_inferred_axiom(&mut self, axiom: Axiom) -> Result<AxiomId> {
        self.validate_inverse_pair(&axiom)?;
        let id = self.axioms.push_inferred(axiom, &self.entities)?;
        let stored = self.axioms.get(id)?;
        self.index.insert(id, stored);
        self.revision.bump();
        Ok(id)
    }

    /// Entity signature for a stored axiom.
    pub fn signature_of_axiom(&self, id: AxiomId) -> Result<std::collections::HashSet<EntityId>> {
        let axiom = self.axioms.get(id)?;
        Ok(axiom_signature(axiom))
    }

    /// Union of entity signatures for all dirty added axioms.
    #[must_use]
    pub fn dirty_signatures(&self) -> std::collections::HashSet<EntityId> {
        let mut sig = std::collections::HashSet::new();
        for id in self.dirty.added() {
            if let Ok(axiom) = self.axioms.get(*id) {
                sig.extend(axiom_signature(axiom));
            }
        }
        for id in self.dirty.removed() {
            if let Ok(axiom) = self.axioms.get_raw(*id) {
                sig.extend(axiom_signature(axiom));
            }
        }
        sig
    }

    /// Number of unique interned IRIs.
    #[must_use]
    pub fn iri_count(&self) -> usize {
        self.iris.len()
    }

    /// Access the IRI intern pool.
    #[must_use]
    pub fn iris(&self) -> &InternPool {
        &self.iris
    }

    /// Access the entity registry.
    #[must_use]
    pub fn entities(&self) -> &EntityRegistry {
        &self.entities
    }

    /// Access the axiom store.
    #[must_use]
    pub fn axioms(&self) -> &AxiomStore {
        &self.axioms
    }

    /// Access axiom indexes.
    #[must_use]
    pub fn index(&self) -> &AxiomIndex {
        &self.index
    }

    /// Resolve an interned IRI to its string value.
    pub fn resolve_iri(&self, id: IriId) -> Result<&str> {
        self.iris.resolve(id)
    }

    /// Look up an entity by IRI string, registering it if absent.
    pub fn entity_id(&mut self, iri: &str, kind: EntityKind) -> Result<EntityId> {
        let iri_id = self.iris.intern(iri)?;
        let iri_str = self.iris.resolve(iri_id)?;
        self.entities.get_or_register(iri_id, iri_str, kind)
    }

    /// Look up an entity id by IRI string, validating the IRI format.
    ///
    /// Returns `Err([Error::InvalidIri](crate::Error::InvalidIri))` for malformed IRIs,
    /// `Ok(None)` if the IRI is valid but not registered, or `Ok(Some(id))` on success.
    pub fn try_lookup_entity(&self, iri: &str) -> Result<Option<EntityId>> {
        validate_iri(iri)?;
        Ok(self
            .iris
            .get(iri)
            .and_then(|iri_id| self.entities.entity_by_iri(iri_id)))
    }

    /// Look up an entity id by IRI string without registering.
    ///
    /// Returns `None` for invalid IRIs or unknown entities.
    #[must_use]
    pub fn lookup_entity(&self, iri: &str) -> Option<EntityId> {
        self.try_lookup_entity(iri).ok().flatten()
    }

    /// Get an entity record by id.
    pub fn entity(&self, id: EntityId) -> Result<&EntityRecord> {
        self.entities.entity(id)
    }

    /// Get an axiom by id.
    pub fn axiom(&self, id: AxiomId) -> Result<&Axiom> {
        self.axioms.get(id)
    }

    /// Direct declared superclasses of a class.
    #[must_use]
    pub fn direct_superclasses(&self, class: EntityId) -> &[EntityId] {
        self.index.direct_superclasses(class)
    }

    /// Direct declared subclasses of a class.
    #[must_use]
    pub fn direct_subclasses(&self, class: EntityId) -> &[EntityId] {
        self.index.direct_subclasses(class)
    }

    /// Direct declared super-properties of a property.
    #[must_use]
    pub fn direct_superproperties(&self, property: EntityId) -> &[EntityId] {
        self.index.direct_superproperties(property)
    }

    /// Direct declared sub-properties of a property.
    #[must_use]
    pub fn direct_subproperties(&self, property: EntityId) -> &[EntityId] {
        self.index.direct_subproperties(property)
    }

    /// Declared equivalent classes for a class.
    #[must_use]
    pub fn equivalents_of(&self, class: EntityId) -> Option<&std::collections::HashSet<EntityId>> {
        self.index.equivalents_of(class)
    }

    /// Declared disjoint classes for a class.
    #[must_use]
    pub fn disjoint_with(&self, class: EntityId) -> Option<&std::collections::HashSet<EntityId>> {
        self.index.disjoint_with(class)
    }

    /// Declared inverse object property, if any.
    #[must_use]
    pub fn inverse_of(&self, property: EntityId) -> Option<EntityId> {
        self.index.inverse_of(property)
    }

    /// Existential restrictions declared for a subclass (`property`, `filler` pairs).
    #[must_use]
    pub fn existentials_of(&self, subclass: EntityId) -> &[(EntityId, EntityId)] {
        self.index.existentials_of(subclass)
    }

    /// Classes asserted for an individual.
    #[must_use]
    pub fn classes_of(&self, individual: EntityId) -> &[EntityId] {
        self.index.classes_of(individual)
    }

    /// Individuals asserted for a class.
    #[must_use]
    pub fn individuals_of(&self, class: EntityId) -> &[EntityId] {
        self.index.individuals_of(class)
    }

    /// Object property assertions with the given subject.
    #[must_use]
    pub fn object_assertions_of(&self, subject: EntityId) -> &[(EntityId, EntityId)] {
        self.index.object_assertions_of(subject)
    }

    /// Object property assertions with the given object (`property`, `subject` pairs).
    #[must_use]
    pub fn object_assertions_to(&self, object: EntityId) -> &[(EntityId, EntityId)] {
        self.index.object_assertions_to(object)
    }

    /// Declared equivalent object properties.
    #[must_use]
    pub fn equivalent_properties_of(
        &self,
        property: EntityId,
    ) -> Option<&std::collections::HashSet<EntityId>> {
        self.index.equivalent_properties_of(property)
    }

    /// Individuals declared `sameAs` the given individual.
    #[must_use]
    pub fn same_as(&self, individual: EntityId) -> Option<&std::collections::HashSet<EntityId>> {
        self.index.same_as(individual)
    }

    /// Individuals declared `differentFrom` the given individual.
    #[must_use]
    pub fn different_from(
        &self,
        individual: EntityId,
    ) -> Option<&std::collections::HashSet<EntityId>> {
        self.index.different_from(individual)
    }

    /// Add a validated axiom, updating indexes.
    pub fn add_axiom(&mut self, axiom: Axiom) -> Result<AxiomId> {
        self.validate_inverse_pair(&axiom)?;
        let id = self.axioms.push(axiom, &self.entities)?;
        let stored = self.axioms.get(id)?;
        self.index.insert(id, stored);
        self.revision.bump();
        self.dirty.record_add(id);
        Ok(id)
    }

    /// Remove an axiom by id (tombstone). Rebuilds indexes.
    pub fn remove_axiom(&mut self, id: AxiomId) -> Result<()> {
        self.axioms.get(id)?;
        self.axioms.remove(id)?;
        self.index.rebuild_from_store(&self.axioms);
        self.revision.bump();
        self.dirty.record_remove(id);
        Ok(())
    }

    /// Intern an IRI without registering an entity.
    pub fn intern_iri(&mut self, iri: &str) -> Result<IriId> {
        self.iris.intern(iri)
    }

    fn validate_inverse_pair(&self, axiom: &Axiom) -> Result<()> {
        let Axiom::InverseObjectProperties { left, right } = axiom else {
            return Ok(());
        };
        if let Some(existing) = self.index.inverse_of(*left) {
            if existing != *right {
                return Err(Error::InvalidAxiom(format!(
                    "property {left:?} already has inverse {existing:?}, cannot add inverse {right:?}"
                )));
            }
        }
        if let Some(existing) = self.index.inverse_of(*right) {
            if existing != *left {
                return Err(Error::InvalidAxiom(format!(
                    "property {right:?} already has inverse {existing:?}, cannot add inverse {left:?}"
                )));
            }
        }
        Ok(())
    }
}

/// Fluent builder for constructing ontologies in memory.
#[derive(Debug, Default)]
pub struct OntologyBuilder {
    ontology: Ontology,
}

impl OntologyBuilder {
    /// Create a new builder with an empty ontology.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a class entity.
    pub fn class(mut self, iri: &str) -> Result<Self> {
        self.ontology.entity_id(iri, EntityKind::Class)?;
        Ok(self)
    }

    /// Register an individual entity.
    pub fn individual(mut self, iri: &str) -> Result<Self> {
        self.ontology.entity_id(iri, EntityKind::Individual)?;
        Ok(self)
    }

    /// Register an object property entity.
    pub fn object_property(mut self, iri: &str) -> Result<Self> {
        self.ontology.entity_id(iri, EntityKind::ObjectProperty)?;
        Ok(self)
    }

    /// Add a `SubClassOf` axiom.
    pub fn subclass_of(mut self, subclass: &str, superclass: &str) -> Result<Self> {
        let sub = self.ontology.entity_id(subclass, EntityKind::Class)?;
        let sup = self.ontology.entity_id(superclass, EntityKind::Class)?;
        self.ontology.add_axiom(Axiom::SubClassOf {
            subclass: sub,
            superclass: sup,
        })?;
        Ok(self)
    }

    /// Add a `SubObjectPropertyOf` axiom.
    pub fn subproperty_of(mut self, sub: &str, sup: &str) -> Result<Self> {
        let sub_id = self.ontology.entity_id(sub, EntityKind::ObjectProperty)?;
        let sup_id = self.ontology.entity_id(sup, EntityKind::ObjectProperty)?;
        self.ontology.add_axiom(Axiom::SubObjectPropertyOf {
            sub_property: sub_id,
            super_property: sup_id,
        })?;
        Ok(self)
    }

    /// Add an `ObjectPropertyDomain` axiom.
    pub fn property_domain(mut self, property: &str, domain: &str) -> Result<Self> {
        let property_id = self
            .ontology
            .entity_id(property, EntityKind::ObjectProperty)?;
        let domain_id = self.ontology.entity_id(domain, EntityKind::Class)?;
        self.ontology.add_axiom(Axiom::ObjectPropertyDomain {
            property: property_id,
            domain: domain_id,
        })?;
        Ok(self)
    }

    /// Add an `ObjectPropertyRange` axiom.
    pub fn property_range(mut self, property: &str, range: &str) -> Result<Self> {
        let property_id = self
            .ontology
            .entity_id(property, EntityKind::ObjectProperty)?;
        let range_id = self.ontology.entity_id(range, EntityKind::Class)?;
        self.ontology.add_axiom(Axiom::ObjectPropertyRange {
            property: property_id,
            range: range_id,
        })?;
        Ok(self)
    }

    /// Add a `ClassAssertion` axiom.
    pub fn class_assertion(mut self, individual: &str, class: &str) -> Result<Self> {
        let individual_id = self
            .ontology
            .entity_id(individual, EntityKind::Individual)?;
        let class_id = self.ontology.entity_id(class, EntityKind::Class)?;
        self.ontology.add_axiom(Axiom::ClassAssertion {
            individual: individual_id,
            class: class_id,
        })?;
        Ok(self)
    }

    /// Add an `ObjectPropertyAssertion` axiom.
    pub fn object_property_assertion(
        mut self,
        subject: &str,
        property: &str,
        object: &str,
    ) -> Result<Self> {
        let subject_id = self.ontology.entity_id(subject, EntityKind::Individual)?;
        let property_id = self
            .ontology
            .entity_id(property, EntityKind::ObjectProperty)?;
        let object_id = self.ontology.entity_id(object, EntityKind::Individual)?;
        self.ontology.add_axiom(Axiom::ObjectPropertyAssertion {
            subject: subject_id,
            property: property_id,
            object: object_id,
        })?;
        Ok(self)
    }

    /// Add a `SameIndividual` axiom.
    pub fn same_individual(mut self, individuals: &[&str]) -> Result<Self> {
        let ids = individuals
            .iter()
            .map(|iri| self.ontology.entity_id(iri, EntityKind::Individual))
            .collect::<Result<Vec<_>>>()?;
        self.ontology.add_axiom(Axiom::SameIndividual(ids))?;
        Ok(self)
    }

    /// Add a `DifferentIndividuals` axiom.
    pub fn different_individuals(mut self, individuals: &[&str]) -> Result<Self> {
        let ids = individuals
            .iter()
            .map(|iri| self.ontology.entity_id(iri, EntityKind::Individual))
            .collect::<Result<Vec<_>>>()?;
        self.ontology.add_axiom(Axiom::DifferentIndividuals(ids))?;
        Ok(self)
    }

    /// Add an `EquivalentObjectProperties` axiom.
    pub fn equivalent_object_properties(mut self, properties: &[&str]) -> Result<Self> {
        let ids = properties
            .iter()
            .map(|iri| self.ontology.entity_id(iri, EntityKind::ObjectProperty))
            .collect::<Result<Vec<_>>>()?;
        self.ontology
            .add_axiom(Axiom::EquivalentObjectProperties(ids))?;
        Ok(self)
    }

    /// Declare an asymmetric object property.
    pub fn asymmetric_object_property(mut self, property: &str) -> Result<Self> {
        let property_id = self
            .ontology
            .entity_id(property, EntityKind::ObjectProperty)?;
        self.ontology
            .add_axiom(Axiom::AsymmetricObjectProperty(property_id))?;
        Ok(self)
    }

    /// Build the ontology.
    pub fn build(self) -> Result<Ontology> {
        Ok(self.ontology)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_constructs_taxonomy() {
        let ontology = Ontology::builder()
            .class("http://example.org/A")
            .expect("class A")
            .class("http://example.org/B")
            .expect("class B")
            .subclass_of("http://example.org/A", "http://example.org/B")
            .expect("subclass")
            .build()
            .expect("build");

        let a = ontology.lookup_entity("http://example.org/A").expect("A");
        let b = ontology.lookup_entity("http://example.org/B").expect("B");
        assert_eq!(ontology.direct_superclasses(a), &[b]);
        assert_eq!(ontology.direct_subclasses(b), &[a]);
    }

    #[test]
    fn from_file_returns_parse_not_available() {
        let err = Ontology::from_file("any.owl").expect_err("should fail");
        assert_eq!(err, Error::ParseNotAvailable);
    }

    #[test]
    fn try_lookup_entity_rejects_invalid_iri() {
        let ontology = Ontology::new();
        let err = ontology
            .try_lookup_entity("relative/path")
            .expect_err("invalid");
        assert!(matches!(err, Error::InvalidIri(_)));
    }
}
