use thiserror::Error;

use crate::entity::{EntityId, EntityKind};

/// Result type alias using [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Errors produced by the core ontology model.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Error {
    /// The IRI string is not a valid absolute IRI.
    #[error("invalid IRI: {0}")]
    InvalidIri(String),
    /// An entity was registered with a conflicting kind for the same IRI.
    #[error("entity kind mismatch for {iri}: expected {expected:?}, found {found:?}")]
    EntityKindMismatch {
        /// IRI identifier for the conflicting entity.
        iri: String,
        /// Expected entity kind.
        expected: EntityKind,
        /// Found entity kind.
        found: EntityKind,
    },
    /// Referenced entity id does not exist.
    #[error("unknown entity: {0:?}")]
    UnknownEntity(EntityId),
    /// Axiom validation failed.
    #[error("invalid axiom: {0}")]
    InvalidAxiom(String),
    /// File parsing is not available until v0.2.
    #[error("ontology file parsing is not available until v0.2; use Ontology::from_json or the builder API")]
    ParseNotAvailable,
    /// Serialization or deserialization failed.
    #[error("serialization error: {0}")]
    Serialization(String),
    /// Ontology not loaded (reasoner).
    #[error("ontology not loaded")]
    OntologyNotLoaded,
    /// Reasoning not yet implemented.
    #[error("reasoning not yet implemented")]
    NotImplemented,
    /// Generic error with message.
    #[error("{0}")]
    Message(String),
}
