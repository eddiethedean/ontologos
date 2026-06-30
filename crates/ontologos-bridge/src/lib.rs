//! Model adapters between `ontologos-core`, horned-owl, and reasonable.

mod error;
mod horned;
mod perf;
mod reasonable_session;
mod rl_postprocess;
mod taxonomy;
mod triples;

pub use error::{Error, Result};
pub use horned::core_to_horned;
pub use perf::{BridgePerfTimings, perf_enabled as bridge_perf_enabled};
pub use reasonable_session::{
    MaterializeOutcome, ReasonableSession, downcast_reasonable_session, materialize_with_session,
    take_reasonable_session,
};
pub use rl_postprocess::{
    apply_characteristic_propagation, apply_domain_range_inheritance,
    apply_equivalent_property_subproperties, apply_existential_subclass_subsumption,
    apply_inverse_subproperty_materialization, apply_rdfs_fallbacks, apply_reasonable_fallbacks,
    apply_transitive_subproperties, has_bottom_chain_violation,
};
pub use taxonomy::{equivalence_clusters, reduce_subsumptions};
pub use triples::{
    MergeLimits, MergeReport, core_to_triples, core_to_triples_all, core_to_triples_for_axioms,
    merge_triples_into_ontology, merge_triples_into_ontology_with_limits,
};
