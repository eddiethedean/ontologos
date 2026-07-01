//! RDFS reasoning facade over [`reasonable`](https://crates.io/crates/reasonable).

mod engine;
mod profile_engine;
mod reasoner;
mod report;

pub use engine::RdfsEngine;
pub use profile_engine::RdfsEngineAdapter;
pub use reasoner::{classify_reasoner, materialize_reasoner, materialize_routed};
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
    #[error(transparent)]
    Bridge(#[from] ontologos_bridge::Error),
    #[error(transparent)]
    Reasonable(#[from] reasonable::error::ReasonableError),
}
