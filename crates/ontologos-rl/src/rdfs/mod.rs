//! RDFS materialization via reasonable (subset of OWL RL rules).

mod engine;
mod reasoner;
mod report;

pub use engine::RdfsEngine;
pub use reasoner::{classify_reasoner, materialize_reasoner, materialize_routed};
pub use report::{InferenceRecord, MaterializationReport, RdfsRule};

pub use crate::Error;
pub type Result<T> = crate::Result<T>;
