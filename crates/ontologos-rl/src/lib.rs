//! OWL RL forward-chaining rule engine.
//!
//! # Example
//!
//! ```
//! use ontologos_core::Ontology;
//! use ontologos_rl::RlEngine;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut ontology = Ontology::builder()
//!     .class("http://example.org/A")?
//!     .class("http://example.org/B")?
//!     .subclass_of("http://example.org/A", "http://example.org/B")?
//!     .build()?;
//! let report = RlEngine::new(1).saturate(&mut ontology)?;
//! assert!(report.inferred_total() >= 0);
//! # Ok(())
//! # }
//! ```

mod engine;
mod reasoner;
mod report;
mod rules;
mod triple_index;

pub use engine::RlEngine;
pub use reasoner::{classify_reasoner, materialize_reasoner};
pub use report::{InferenceRecord, MaterializationReport, RlRule};
pub use triple_index::TripleIndex;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_new_rejects_invalid_parallelism() {
        assert!(RlEngine::try_new(0).is_err());
        assert!(RlEngine::try_new(65).is_err());
        assert!(RlEngine::try_new(1).is_ok());
    }
}
