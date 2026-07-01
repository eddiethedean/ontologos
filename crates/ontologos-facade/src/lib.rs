//! Unified reasoner facade — routes all OWL profiles without circular crate deps.

#![warn(missing_docs)]

mod classify;
mod engines;
mod entailment;
mod error;
mod json;
mod lookup;
mod query;

pub use classify::{classify, taxonomy_from_outcome};
pub use entailment::{
    check_consistency, is_consistent, is_entailed, is_entailed_axiom, is_subsumption_entailed,
};
pub use error::{EntailmentCheck, Error, Result};
pub use lookup::{get_object_property_values, get_sub_object_properties};
pub use ontologos_core::ConsistencyResult;
pub use ontologos_el::ClassifyOutcome;
pub use json::{rdfs_materialization_json, rl_materialization_json, taxonomy_json};
pub use query::{taxonomy_hierarchy, query_engine};
