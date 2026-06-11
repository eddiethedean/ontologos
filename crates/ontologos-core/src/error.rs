use thiserror::Error;

use crate::entity::{EntityId, EntityKind};

/// Result type alias using [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Errors produced by the core ontology model.
///
/// See the [error reference](https://github.com/eddiethedean/ontologos/blob/main/docs/reference/errors.md)
/// for recovery guidance.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Error {
    /// The IRI string is not a valid absolute IRI (wrong scheme, control chars, too long).
    ///
    /// Use `http`, `https`, or `urn` schemes only.
    #[error("invalid IRI: {0}")]
    InvalidIri(String),
    /// An entity was registered with a conflicting kind for the same IRI.
    #[error("entity kind mismatch for {iri}: expected {expected:?}, found {found:?}")]
    EntityKindMismatch {
        /// Resolved IRI string for the conflicting entity.
        iri: String,
        /// Expected entity kind.
        expected: EntityKind,
        /// Found entity kind.
        found: EntityKind,
    },
    /// Referenced entity id does not exist in the registry.
    #[error("unknown entity: {0:?}")]
    UnknownEntity(EntityId),
    /// Axiom validation failed (wrong kinds, unknown IRI in JSON, degenerate operands).
    #[error("invalid axiom: {0}")]
    InvalidAxiom(String),
    /// File parsing is not available until v0.2.
    ///
    /// Use [`Ontology::from_json`](crate::Ontology::from_json) or the builder API instead.
    #[error("ontology file parsing is not available until v0.2; use Ontology::from_json or the builder API")]
    ParseNotAvailable,
    /// Serialization or deserialization failed (bad JSON, limits, format version).
    #[error("serialization error: {0}")]
    Serialization(String),
    /// Ontology not loaded (reasoner).
    #[error("ontology not loaded")]
    OntologyNotLoaded,
    /// Reasoning not yet implemented for the selected engine or profile.
    #[error("reasoning not yet implemented")]
    NotImplemented,
    /// Generic configuration or validation error.
    #[error("{0}")]
    Message(String),
}
