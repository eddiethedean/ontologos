//! Model adapters between `ontologos-core`, horned-owl, and reasonable.

mod error;
mod horned;
mod reasonable_session;
mod rl_postprocess;
mod taxonomy;
mod triples;

pub use error::{Error, Result};
pub use horned::core_to_horned;
pub use reasonable_session::{
    downcast_reasonable_session, materialize_with_session, take_reasonable_session,
    MaterializeOutcome, ReasonableSession,
};
pub use rl_postprocess::{
    apply_domain_range_inheritance, apply_reasonable_fallbacks, apply_transitive_subproperties,
};
pub use taxonomy::{equivalence_clusters, reduce_subsumptions};
pub use triples::{
    core_to_triples, core_to_triples_all, core_to_triples_for_axioms, merge_triples_into_ontology,
    merge_triples_into_ontology_with_limits, MergeLimits, MergeReport,
};
