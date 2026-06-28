//! OWL QL conjunctive query answering over classified taxonomies.

#![warn(missing_docs)]

mod parse;
mod query;
pub mod rewrite;

use ontologos_core::{EntityId, Ontology, Taxonomy};
use ontologos_query::QueryEngine;
use thiserror::Error;

pub use parse::parse_conjunctive_query;
pub use query::{ConjunctiveQuery, QueryAnswer, QueryAtom};
pub use rewrite::{is_ql_shape, rewrite_query};

/// Result type for QL operations.
pub type Result<T> = std::result::Result<T, Error>;

/// QL query errors.
#[derive(Debug, Error)]
pub enum Error {
    /// Unknown class in query.
    #[error("unknown class in query: {0}")]
    UnknownClass(String),
    /// Parse error.
    #[error("query parse error: {0}")]
    Parse(String),
    /// Query engine error.
    #[error(transparent)]
    Query(#[from] ontologos_query::Error),
}

/// Answer a conjunctive query over a classified ontology.
pub fn answer_query<'a>(
    ontology: &'a Ontology,
    taxonomy: &'a Taxonomy,
    query: &ConjunctiveQuery,
) -> Result<Vec<QueryAnswer>> {
    let engine = QueryEngine::new(ontology, taxonomy);
    query::evaluate(&engine, ontology, query)
}

/// Check class subsumption entailment (QL fragment).
pub fn is_entailed(
    ontology: &Ontology,
    taxonomy: &Taxonomy,
    sub: EntityId,
    sup: EntityId,
) -> Result<bool> {
    let engine = QueryEngine::new(ontology, taxonomy);
    engine.is_subsumed(sub, sup).map_err(Error::from)
}
