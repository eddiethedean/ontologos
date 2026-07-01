//! OWL RL forward-chaining facade over [`reasonable`](https://crates.io/crates/reasonable).
//!
//! # Start here — load a file and saturate
//!
//! ```no_run
//! use ontologos_parser::load_ontology;
//! use ontologos_rl::RlEngine;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut ontology = load_ontology(std::path::Path::new("ontology.owl"))?;
//! let report = RlEngine::new(1).saturate(&mut ontology)?;
//! println!("inferred {}", report.inferred_total());
//! # Ok(())
//! # }
//! ```

mod engine;
mod native_eval;
mod profile_engine;
mod reasoner;
mod report;

pub use engine::RlEngine;
pub use native_eval::transitive_subclass_closure;
pub use profile_engine::RlEngineAdapter;
pub use reasoner::{classify_reasoner, materialize_reasoner, saturate_routed};
pub use report::{InferenceRecord, MaterializationReport, RlRule};

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
