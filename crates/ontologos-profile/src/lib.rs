//! OWL profile detection and diagnostics.

use ontologos_core::Ontology;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("profile detection not yet implemented")]
    NotImplemented,
}

/// Detected OWL 2 profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// Detect the most specific OWL profile supported by the ontology.
pub fn detect_profile(ontology: &Ontology) -> Result<ProfileReport> {
    let _ = ontology;
    Ok(ProfileReport {
        detected: None,
        diagnostics: Vec::new(),
    })
}
