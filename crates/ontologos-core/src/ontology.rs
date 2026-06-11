use std::path::Path;

use crate::axiom::{Axiom, AxiomId};
use crate::entity::{EntityId, EntityKind, EntityRecord, EntityRegistry};
use crate::error::{Error, Result};
use crate::graph::{AxiomIndex, AxiomStore};
use crate::iri::{InternPool, IriId};

/// In-memory ontology with interned IRIs, typed entities, and indexed axioms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ontology {
    pub(crate) iris: InternPool,
    pub(crate) entities: EntityRegistry,
    pub(crate) axioms: AxiomStore,
    pub(crate) index: AxiomIndex,
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
        }
    }

    /// Create a builder for programmatic ontology construction.
    #[must_use]
    pub fn builder() -> OntologyBuilder {
        OntologyBuilder::new()
    }

    /// Load an ontology from a file path.
    ///
    /// File parsing is available in v0.2 via `ontologos-parser`.
    pub fn from_file(_path: impl AsRef<Path>) -> Result<Self> {
        Err(Error::ParseNotAvailable)
    }

    /// Number of registered entities.
    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Number of stored axioms.
    #[must_use]
    pub fn axiom_count(&self) -> usize {
        self.axioms.len()
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
        self.entities.get_or_register(iri_id, kind)
    }

    /// Look up an entity id by IRI string without registering.
    #[must_use]
    pub fn lookup_entity(&self, iri: &str) -> Option<EntityId> {
        let iri_id = self.iris.get(iri)?;
        self.entities.entity_by_iri(iri_id)
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

    /// Add a validated axiom, updating indexes.
    pub fn add_axiom(&mut self, axiom: Axiom) -> Result<AxiomId> {
        let id = self.axioms.push(axiom, &self.entities)?;
        let stored = self.axioms.get(id)?;
        self.index.insert(id, stored);
        Ok(id)
    }

    /// Intern an IRI without registering an entity.
    pub fn intern_iri(&mut self, iri: &str) -> Result<IriId> {
        self.iris.intern(iri)
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

        assert_eq!(ontology.entity_count(), 2);
        assert_eq!(ontology.axiom_count(), 1);
    }

    #[test]
    fn from_file_returns_parse_not_available() {
        let err = Ontology::from_file("any.owl").expect_err("should fail");
        assert_eq!(err, Error::ParseNotAvailable);
    }
}
