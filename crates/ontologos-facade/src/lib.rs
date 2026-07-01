//! Unified reasoner facade — routes all OWL profiles without circular crate deps.

#![warn(missing_docs)]

mod classify;
mod engines;
mod entailment;
mod error;
mod lookup;
mod query;

pub use classify::{classify, taxonomy_from_outcome};
pub use entailment::{is_consistent, is_entailed, is_entailed_axiom, is_subsumption_entailed};
pub use error::{EntailmentCheck, Error, Result};
pub use lookup::{get_object_property_values, get_sub_object_properties};
pub use ontologos_el::ClassifyOutcome;
pub use query::query_engine;
