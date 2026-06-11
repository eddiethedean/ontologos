use serde::{Deserialize, Serialize};

/// Stable identifier for an ontology entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u32);

/// Kind of entity stored in the ontology registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityKind {
    Class,
    Individual,
    ObjectProperty,
    DataProperty,
    AnnotationProperty,
}
