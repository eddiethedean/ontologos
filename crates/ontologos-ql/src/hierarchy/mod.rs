//! Taxonomy hierarchy navigation (subclasses, subsumption) after classification.

mod graph;

use ontologos_core::{EntityId, Ontology, Taxonomy};
use thiserror::Error;

pub use graph::TaxonomyGraph;

/// Result type for hierarchy navigation.
pub type Result<T> = std::result::Result<T, Error>;

/// Hierarchy navigation errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error("unknown entity {0:?}")]
    UnknownEntity(EntityId),
    #[error(transparent)]
    Core(#[from] ontologos_core::Error),
}

/// Navigate a classified taxonomy (not OWL QL conjunctive queries).
#[derive(Debug)]
pub struct TaxonomyHierarchy<'a> {
    ontology: &'a Ontology,
    taxonomy: &'a Taxonomy,
    graph: TaxonomyGraph,
}

/// Legacy name for [`TaxonomyHierarchy`].
pub type QueryEngine<'a> = TaxonomyHierarchy<'a>;

impl<'a> TaxonomyHierarchy<'a> {
    #[must_use]
    pub fn new(ontology: &'a Ontology, taxonomy: &'a Taxonomy) -> Self {
        Self {
            ontology,
            taxonomy,
            graph: TaxonomyGraph::from_taxonomy(taxonomy),
        }
    }

    #[must_use]
    pub fn ontology(&self) -> &'a Ontology {
        self.ontology
    }

    #[must_use]
    pub fn taxonomy(&self) -> &'a Taxonomy {
        self.taxonomy
    }

    pub fn direct_subclasses(&self, class: EntityId) -> Result<Vec<EntityId>> {
        self.ensure_class(class)?;
        Ok(self.graph.direct_subclasses(class))
    }

    pub fn direct_superclasses(&self, class: EntityId) -> Result<Vec<EntityId>> {
        self.ensure_class(class)?;
        Ok(self.graph.direct_superclasses(class))
    }

    pub fn is_subsumed(&self, sub: EntityId, sup: EntityId) -> Result<bool> {
        self.ensure_class(sub)?;
        self.ensure_class(sup)?;
        Ok(self.graph.is_subsumed(sub, sup))
    }

    pub fn lookup(&self, iri: &str) -> Option<EntityId> {
        self.ontology.lookup_entity(iri)
    }

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
            if *asserted == class || self.graph.is_subsumed(*asserted, class) {
                out.push(*individual);
            }
        }
        out.sort_by_key(|id| id.0);
        out.dedup();
        Ok(out)
    }

    pub fn types_of(&self, individual: EntityId) -> Result<Vec<EntityId>> {
        let record = self.ontology.entity(individual)?;
        if record.kind != ontologos_core::EntityKind::Individual {
            return Err(Error::UnknownEntity(individual));
        }
        let mut out = Vec::new();
        for (_, axiom) in self.ontology.axioms().iter() {
            if let ontologos_core::Axiom::ClassAssertion {
                individual: subj,
                class,
            } = axiom
            {
                if *subj == individual {
                    out.push(*class);
                }
            }
        }
        Ok(out)
    }

    fn ensure_class(&self, class: EntityId) -> Result<()> {
        let record = self.ontology.entity(class)?;
        if record.kind != ontologos_core::EntityKind::Class {
            return Err(Error::UnknownEntity(class));
        }
        Ok(())
    }
}
