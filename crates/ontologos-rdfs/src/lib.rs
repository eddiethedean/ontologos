//! RDFS reasoning via graph closure and property propagation.
//!
//! # Example
//!
//! ```
//! use ontologos_core::Ontology;
//! use ontologos_rdfs::RdfsEngine;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut ontology = Ontology::builder()
//!     .class("http://example.org/A")?
//!     .class("http://example.org/B")?
//!     .class("http://example.org/C")?
//!     .subclass_of("http://example.org/A", "http://example.org/B")?
//!     .subclass_of("http://example.org/B", "http://example.org/C")?
//!     .build()?;
//! let report = RdfsEngine::new().materialize(&mut ontology)?;
//! assert!(report.inferred_total() >= 1);
//! # Ok(())
//! # }
//! ```

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
