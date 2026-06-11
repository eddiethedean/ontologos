use std::path::Path;

use crate::axiom::AxiomKind;
use crate::entity::{EntityId, EntityKind};
use crate::error::{Error, Result};

/// In-memory ontology graph with interned IRIs and axiom storage.
#[derive(Debug, Default)]
pub struct Ontology {
    entities: Vec<EntityKind>,
    axioms: Vec<AxiomKind>,
}

impl Ontology {
    /// Load an ontology from a file path.
    ///
    /// Parsing is delegated to `ontologos-parser` in a future release.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(Error::Message(format!(
                "file not found: {}",
                path.display()
            )));
        }
        Ok(Self::default())
    }

    /// Register a new entity and return its stable identifier.
    pub fn register_entity(&mut self, kind: EntityKind) -> EntityId {
        let id = EntityId(u32::try_from(self.entities.len()).expect("entity id overflow"));
        self.entities.push(kind);
        id
    }

    /// Append an axiom to the ontology.
    pub fn add_axiom(&mut self, axiom: AxiomKind) {
        self.axioms.push(axiom);
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
}
