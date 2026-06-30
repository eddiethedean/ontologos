//! Query interface over classified ontologies (petgraph-backed hierarchy views).

mod graph;

use ontologos_core::{EntityId, Ontology, Taxonomy};
use thiserror::Error;

pub use graph::TaxonomyGraph;

/// Result type for query operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Query errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error("unknown entity {0:?}")]
    UnknownEntity(EntityId),
    #[error(transparent)]
    Core(#[from] ontologos_core::Error),
}

/// Query handle over a classified ontology taxonomy.
#[derive(Debug)]
pub struct QueryEngine<'a> {
    ontology: &'a Ontology,
    taxonomy: &'a Taxonomy,
    graph: TaxonomyGraph,
}

impl<'a> QueryEngine<'a> {
    #[must_use]
    pub fn new(ontology: &'a Ontology, taxonomy: &'a Taxonomy) -> Self {
        Self {
            ontology,
            taxonomy,
            graph: TaxonomyGraph::from_taxonomy(taxonomy),
        }
    }

    /// Underlying ontology reference.
    #[must_use]
    pub fn ontology(&self) -> &'a Ontology {
        self.ontology
    }

    /// Classified taxonomy reference.
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

    pub fn equivalent_classes(&self, class: EntityId) -> Result<Option<Vec<EntityId>>> {
        self.ensure_class(class)?;
        Ok(self
            .taxonomy
            .equivalent_classes(class)
            .map(<[EntityId]>::to_vec))
    }

    pub fn unsatisfiable_classes(&self) -> Vec<EntityId> {
        self.taxonomy.unsatisfiable.clone()
    }

    /// Whether `sub` is entailed to be subsumed by `sup` (taxonomy + asserted edges).
    pub fn is_entailed(&self, sub: EntityId, sup: EntityId) -> Result<bool> {
        self.is_subsumed(sub, sup)
    }

    pub fn lookup(&self, iri: &str) -> Option<EntityId> {
        self.ontology.lookup_entity(iri)
    }

    /// Named individuals with an entailed `ClassAssertion` to `class` (direct or via subsumption).
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

    /// Asserted classes for a named individual (no inference).
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

#[cfg(test)]
mod tests {
    use ontologos_core::{Axiom, EntityKind, Ontology, Taxonomy};

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

        let taxonomy = Taxonomy::from_parts(vec![(a, b)], vec![], vec![]);
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

        let taxonomy = Taxonomy::from_parts(vec![(a, b), (b, c)], vec![], vec![]);
        let engine = QueryEngine::new(&ontology, &taxonomy);
        assert!(engine.is_subsumed(a, c).expect("subsumed"));
    }

    #[test]
    fn instances_of_entailed_typing() {
        let mut ontology = Ontology::new();
        let a = ontology
            .entity_id("http://ex.org/A", EntityKind::Class)
            .unwrap();
        let b = ontology
            .entity_id("http://ex.org/B", EntityKind::Class)
            .unwrap();
        let i = ontology
            .entity_id("http://ex.org/i", EntityKind::Individual)
            .unwrap();
        ontology
            .add_axiom(Axiom::ClassAssertion {
                individual: i,
                class: a,
            })
            .unwrap();

        let taxonomy = Taxonomy::from_parts(vec![(a, b)], vec![], vec![]);
        let engine = QueryEngine::new(&ontology, &taxonomy);
        let instances = engine.instances_of(b).expect("instances");
        assert_eq!(instances, vec![i]);
    }
}
