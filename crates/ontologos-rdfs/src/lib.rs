//! RDFS reasoning via graph closure and property propagation.
//!
//! # Start here — load a file and materialize
//!
//! ```no_run
//! use ontologos_parser::load_ontology;
//! use ontologos_rdfs::RdfsEngine;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut ontology = load_ontology(std::path::Path::new("ontology.owl"))?;
//! let report = RdfsEngine::new().materialize(&mut ontology)?;
//! println!("inferred {}", report.inferred_total());
//! # Ok(())
//! # }
//! ```
//!
//! Via [`Reasoner`](ontologos_core::Reasoner): use [`classify_reasoner`] — not [`Reasoner::classify`](ontologos_core::Reasoner::classify).

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
