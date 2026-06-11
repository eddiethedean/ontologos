//! OWL profile detection and diagnostics.

mod construct;
mod detect;
mod rules;
pub mod scanner;

use ontologos_core::Ontology;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use construct::OwlConstruct;
pub use detect::detect_profile;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("profile detection failed: {0}")]
    Message(String),
}

/// Detected OWL 2 profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OwlProfile {
    El,
    Rl,
    Ql,
    Dl,
}

/// Diagnostic emitted when unsupported constructs are encountered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileDiagnostic {
    pub construct: String,
    pub message: String,
}

/// Profile detection report for an ontology.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileReport {
    pub detected: Option<OwlProfile>,
    pub diagnostics: Vec<ProfileDiagnostic>,
}

/// Resolve profile from ontology for reasoner configuration helpers.
#[must_use]
pub fn profile_from_ontology(ontology: &Ontology) -> Option<OwlProfile> {
    detect_profile(ontology).ok().and_then(|r| r.detected)
}
