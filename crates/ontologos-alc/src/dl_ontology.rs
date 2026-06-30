//! DL ontology view over core + clausified TBox.

use ontologos_core::Ontology;

use crate::Error;
use crate::clause::ClauseSet;
use crate::normalize::clausify;

/// OWL 2 DL internal ontology (core entities + CE store + clauses).
#[derive(Debug, Clone)]
pub struct DlOntology {
    core: Ontology,
    clauses: ClauseSet,
}

impl DlOntology {
    /// Build from a core ontology with optional DL store population.
    pub fn from_ontology(ontology: &Ontology) -> Result<Self, Error> {
        let mut core = ontology.clone();
        let clauses = clausify(&mut core)?;
        Ok(Self { core, clauses })
    }

    /// Underlying core ontology (includes merged DL store).
    #[must_use]
    pub fn core(&self) -> &Ontology {
        &self.core
    }

    /// Mutable core for interning derived class expressions during reasoning.
    pub(crate) fn core_mut(&mut self) -> &mut Ontology {
        &mut self.core
    }

    /// Clausified TBox/ABox constraints.
    #[must_use]
    pub fn clauses(&self) -> &ClauseSet {
        &self.clauses
    }
}
