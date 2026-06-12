//! OWL RL forward-chaining rule engine.
//!
//! # Start here — load a file and saturate
//!
//! ```no_run
//! use ontologos_parser::load_ontology;
//! use ontologos_rl::RlEngine;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut ontology = load_ontology(std::path::Path::new("ontology.owl"))?;
//! let report = RlEngine::new(1)?.saturate(&mut ontology)?;
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
