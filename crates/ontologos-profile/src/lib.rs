//! OWL profile detection and diagnostics for OntoLogos.
//!
//! v0.2 classifies ontologies into OWL 2 EL, RL, QL, or DL profiles using
//! mapped TBox constructs. Diagnostics also report constructs observed in the
//! full parse that fall outside the detected profile.
//!
//! # Example
//!
//! ```
//! use ontologos_core::Ontology;
//! use ontologos_profile::{detect_profile, OwlProfile};
//!
//! let ontology = Ontology::default();
//! let report = detect_profile(&ontology)?;
//! assert_eq!(report.detected, Some(OwlProfile::Ql));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! See [profile detection guide](https://github.com/eddiethedean/ontologos/blob/main/docs/guides/profile-detection.md).

#![warn(missing_docs)]

mod construct;
mod detect;
mod rules;
/// Construct scanning helpers for profile detection.
pub mod scanner;

use ontologos_core::Ontology;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use construct::OwlConstruct;
pub use detect::detect_profile;

/// Result type alias for profile operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Profile detection failed.
#[derive(Debug, Error)]
pub enum Error {
    /// Profile detection failed with a message.
    #[error("profile detection failed: {0}")]
    Message(String),
}

/// Detected OWL 2 profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OwlProfile {
    /// OWL 2 EL.
    El,
    /// OWL 2 RL.
    Rl,
    /// OWL 2 QL.
    Ql,
    /// OWL 2 DL (fallback).
    Dl,
}

/// Diagnostic emitted when unsupported constructs are encountered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileDiagnostic {
    /// Construct name (debug representation).
    pub construct: String,
    /// Human-readable explanation.
    pub message: String,
}

/// Profile detection report for an ontology.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileReport {
    /// Most specific detected profile, if any.
    pub detected: Option<OwlProfile>,
    /// Constructs outside the detected profile or mapping gaps.
    pub diagnostics: Vec<ProfileDiagnostic>,
}

/// Resolve profile from ontology for reasoner configuration helpers.
#[must_use]
pub fn profile_from_ontology(ontology: &Ontology) -> Option<OwlProfile> {
    detect_profile(ontology).ok().and_then(|r| r.detected)
}
