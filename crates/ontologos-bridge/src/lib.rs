//! Model adapters between `ontologos-core`, horned-owl, whelk, and reasonable.

mod error;
mod horned;
mod taxonomy;
mod triples;
mod whelk;

pub use error::{Error, Result};
pub use horned::core_to_horned;
pub use taxonomy::{equivalence_clusters, reduce_subsumptions};
pub use triples::{core_to_triples, merge_triples_into_ontology, MergeReport};
pub use whelk::{classify_core, classify_horned};
