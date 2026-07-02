//! Taxonomy hierarchy navigation (subclasses, subsumption) after classification.

use ontologos_core::{EntityId, EntityKind, Ontology, Taxonomy};
use thiserror::Error;

/// Result type for hierarchy navigation.
pub type Result<T> = std::result::Result<T, Error>;

/// Hierarchy navigation errors.
#[derive(Debug, Error)]
pub enum Error {
    /// Entity is missing or not a class.
    #[error("unknown entity {0:?}")]
    UnknownEntity(EntityId),
    /// Underlying core error.
    #[error(transparent)]
    Core(#[from] ontologos_core::Error),
}

/// Navigate a classified taxonomy (not OWL QL conjunctive queries).
#[derive(Debug)]
pub struct TaxonomyHierarchy<'a> {
    ontology: &'a Ontology,
    taxonomy: &'a Taxonomy,
}

impl<'a> TaxonomyHierarchy<'a> {
    /// Build a hierarchy view over `ontology` and `taxonomy`.
    #[must_use]
    pub fn new(ontology: &'a Ontology, taxonomy: &'a Taxonomy) -> Self {
        Self { ontology, taxonomy }
    }

    /// Underlying ontology.
    #[must_use]
    pub fn ontology(&self) -> &'a Ontology {
        self.ontology
    }

    /// Classified taxonomy.
    #[must_use]
    pub fn taxonomy(&self) -> &'a Taxonomy {
        self.taxonomy
    }

    /// Direct named subclasses of `class`.
    pub fn direct_subclasses(&self, class: EntityId) -> Result<Vec<EntityId>> {
        self.ensure_class(class)?;
        Ok(self.taxonomy.direct_subclasses(class))
    }

    /// Direct named superclasses of `class`.
    pub fn direct_superclasses(&self, class: EntityId) -> Result<Vec<EntityId>> {
        self.ensure_class(class)?;
        Ok(self.taxonomy.direct_superclasses(class))
    }

    /// Whether `sub` is subsumed by `sup` in the taxonomy.
    pub fn is_subsumed(&self, sub: EntityId, sup: EntityId) -> Result<bool> {
        self.ensure_class(sub)?;
        self.ensure_class(sup)?;
        Ok(self.taxonomy.is_subsumed(sub, sup))
    }

    /// Resolve a class IRI to an entity id.
    #[must_use]
    pub fn lookup(&self, iri: &str) -> Option<EntityId> {
        self.ontology.lookup_entity(iri)
    }

    /// Named individuals with a class assertion to `class` or a subclass.
    pub fn instances_of(&self, class: EntityId) -> Result<Vec<EntityId>> {
        self.ensure_class(class)?;
        let mut out = Vec::new();
        for (_, axiom) in self.ontology.axioms().iter() {
            let ontologos_core::Axiom::ClassAssertion {
                individual,
                class: asserted,
            } = axiom
            else {
                continue;
            };
            if *asserted == class || self.taxonomy.is_subsumed(*asserted, class) {
                out.push(*individual);
            }
        }
        out.sort_by_key(|id| id.0);
        out.dedup();
        Ok(out)
    }

    /// Named classes asserted on `individual`.
    pub fn types_of(&self, individual: EntityId) -> Result<Vec<EntityId>> {
        let record = self.ontology.entity(individual)?;
        if record.kind != EntityKind::Individual {
            return Err(Error::UnknownEntity(individual));
        }
        let mut out = Vec::new();
        for (_, axiom) in self.ontology.axioms().iter() {
            if let ontologos_core::Axiom::ClassAssertion {
                individual: subj,
                class,
            } = axiom
                && *subj == individual
            {
                out.push(*class);
            }
        }
        Ok(out)
    }

    fn ensure_class(&self, class: EntityId) -> Result<()> {
        let record = self.ontology.entity(class)?;
        if record.kind != EntityKind::Class {
            return Err(Error::UnknownEntity(class));
        }
        Ok(())
    }
}
