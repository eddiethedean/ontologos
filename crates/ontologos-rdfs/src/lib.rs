//! RDFS reasoning via graph closure and property propagation.

mod engine;
mod reasoner;
mod report;
mod rules;

pub use engine::RdfsEngine;
pub use reasoner::{classify_reasoner, materialize_reasoner};
pub use report::{InferenceRecord, MaterializationReport, RdfsRule};

use ontologos_core::Error as CoreError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("expected profile {expected:?}, got {actual:?}")]
    WrongProfile {
        expected: ontologos_core::Profile,
        actual: ontologos_core::Profile,
    },
    #[error(transparent)]
    Core(#[from] CoreError),
}
