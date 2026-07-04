//! Shared JavaScript/WASM binding logic for OntoLogos.

#![allow(missing_docs)]

mod convert;
mod error;
mod ontology;
mod reasoner;

pub use convert::{parse_profile, parse_meta_from_ontology, usize_to_u32};
pub use error::{JsError, Result};
pub use ontology::{JsOntology, JsOntologyBuilder};
pub use reasoner::JsReasoner;

/// Package version aligned with the workspace release.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
