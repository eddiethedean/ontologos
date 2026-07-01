//! OWL QL conjunctive query answering and taxonomy hierarchy navigation.

#![warn(missing_docs)]

pub mod hierarchy;
mod parse;
mod query;
pub mod rewrite;

use ontologos_core::{Ontology, Taxonomy};
use thiserror::Error;

pub use hierarchy::{QueryEngine, TaxonomyGraph, TaxonomyHierarchy};
pub use parse::parse_conjunctive_query;
pub use query::{ConjunctiveQuery, QueryAnswer, QueryAtom};
pub use rewrite::rewrite_query;

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
    Query(#[from] hierarchy::Error),
}

/// Answer a conjunctive query over a classified ontology.
pub fn answer_query<'a>(
    ontology: &'a Ontology,
    taxonomy: &'a Taxonomy,
    query: &ConjunctiveQuery,
) -> Result<Vec<QueryAnswer>> {
    let engine = hierarchy::TaxonomyHierarchy::new(ontology, taxonomy);
    query::evaluate(&engine, ontology, query)
}
