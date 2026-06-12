//! Query interface over classified ontologies.
//!
//! # Example
//!
//! ```
//! use ontologos_core::{EntityId, Ontology, Taxonomy};
//! use ontologos_query::QueryEngine;
//!
//! let ontology = Ontology::default();
//! let taxonomy = Taxonomy::default();
//! let engine = QueryEngine::new(&ontology, &taxonomy);
//! assert!(engine.unsatisfiable_classes().is_empty());
//! ```

#![warn(missing_docs)]

use ontologos_core::{EntityId, Ontology, Taxonomy};
use thiserror::Error;

/// Result type for query operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Query errors.
#[derive(Debug, Error)]
pub enum Error {
    /// Entity id is missing or not a class.
    #[error("unknown entity {0:?}")]
    UnknownEntity(EntityId),
    /// Core ontology error.
    #[error(transparent)]
    Core(#[from] ontologos_core::Error),
}

/// Query handle over a classified ontology taxonomy.
#[derive(Debug)]
pub struct QueryEngine<'a> {
    ontology: &'a Ontology,
    taxonomy: &'a Taxonomy,
}

impl<'a> QueryEngine<'a> {
    /// Create a query engine over `ontology` and its `taxonomy`.
    #[must_use]
    pub fn new(ontology: &'a Ontology, taxonomy: &'a Taxonomy) -> Self {
        Self { ontology, taxonomy }
    }

    /// Return direct subclasses of the given class in the reduced taxonomy.
    pub fn direct_subclasses(&self, class: EntityId) -> Result<Vec<EntityId>> {
        self.ensure_class(class)?;
        Ok(self.taxonomy.direct_subclasses(class))
    }

    /// Return direct superclasses of the given class in the reduced taxonomy.
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

    /// Return the equivalence cluster containing `class`, if any.
    pub fn equivalent_classes(&self, class: EntityId) -> Result<Option<Vec<EntityId>>> {
        self.ensure_class(class)?;
        Ok(self
            .taxonomy
            .equivalent_classes(class)
            .map(<[EntityId]>::to_vec))
    }

    /// List classes inferred unsatisfiable (⊥).
    pub fn unsatisfiable_classes(&self) -> Vec<EntityId> {
        self.taxonomy.unsatisfiable.clone()
    }

    /// Resolve an entity IRI to its id.
    pub fn lookup(&self, iri: &str) -> Option<EntityId> {
        self.ontology.lookup_entity(iri)
    }

    fn ensure_class(&self, class: EntityId) -> Result<()> {
        let record = self.ontology.entity(class)?;
        if record.kind != ontologos_core::EntityKind::Class {
            return Err(Error::UnknownEntity(class));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use ontologos_core::{Axiom, EntityKind, Ontology};

    use super::*;

    #[test]
    fn query_direct_subclasses_from_taxonomy() {
        let mut ontology = Ontology::new();
        let a = ontology
            .entity_id("http://ex.org/A", EntityKind::Class)
            .unwrap();
        let b = ontology
            .entity_id("http://ex.org/B", EntityKind::Class)
            .unwrap();
        ontology
            .add_axiom(Axiom::SubClassOf {
                subclass: a,
                superclass: b,
            })
            .unwrap();

        let taxonomy = ontologos_el::ElClassifier::new()
            .classify(&ontology)
            .expect("classify");
        let engine = QueryEngine::new(&ontology, &taxonomy);
        let subs = engine.direct_subclasses(b).expect("subs");
        assert!(subs.contains(&a));
    }

    #[test]
    fn is_subsumed_transitive() {
        let mut ontology = Ontology::new();
        let a = ontology
            .entity_id("http://ex.org/A", EntityKind::Class)
            .unwrap();
        let b = ontology
            .entity_id("http://ex.org/B", EntityKind::Class)
            .unwrap();
        let c = ontology
            .entity_id("http://ex.org/C", EntityKind::Class)
            .unwrap();
        ontology
            .add_axiom(Axiom::SubClassOf {
                subclass: a,
                superclass: b,
            })
            .unwrap();
        ontology
            .add_axiom(Axiom::SubClassOf {
                subclass: b,
                superclass: c,
            })
            .unwrap();

        let taxonomy = ontologos_el::ElClassifier::new()
            .classify(&ontology)
            .expect("classify");
        let engine = QueryEngine::new(&ontology, &taxonomy);
        assert!(engine.is_subsumed(a, c).expect("subsumed"));
    }
}
