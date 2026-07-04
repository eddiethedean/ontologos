//! Typed errors for Node.js bindings.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use ontologos_js::JsError;

#[derive(Debug, Clone, Copy)]
pub enum OntologosStatus {
    ParseError,
    ResourceLimitError,
    IncompleteReasoningError,
    OntologyConflictError,
    Error,
}

impl AsRef<str> for OntologosStatus {
    fn as_ref(&self) -> &str {
        match self {
            Self::ParseError => "ParseError",
            Self::ResourceLimitError => "ResourceLimitError",
            Self::IncompleteReasoningError => "IncompleteReasoningError",
            Self::OntologyConflictError => "OntologyConflictError",
            Self::Error => "Error",
        }
    }
}

fn status_for(error: &JsError) -> OntologosStatus {
    match error.code() {
        "ParseError" => OntologosStatus::ParseError,
        "ResourceLimitError" => OntologosStatus::ResourceLimitError,
        "IncompleteReasoningError" => OntologosStatus::IncompleteReasoningError,
        "OntologyConflictError" => OntologosStatus::OntologyConflictError,
        _ => OntologosStatus::Error,
    }
}

pub(crate) fn map_err(error: JsError) -> Error<OntologosStatus> {
    Error::new(status_for(&error), error.to_string())
}

pub(crate) fn validate_budget_secs(
    budget_secs: Option<u32>,
) -> Result<Option<u32>, OntologosStatus> {
    if budget_secs == Some(0) {
        return Err(map_err(ontologos_js::JsError::Other(
            "budget_secs must be greater than 0; omit for unlimited reasoning".into(),
        )));
    }
    Ok(budget_secs)
}

/// Returns the OntoLogos error code prefix from a Node error message, if present.
#[napi]
pub fn error_code_from_message(message: String) -> Option<String> {
    for code in [
        "ParseError",
        "ResourceLimitError",
        "IncompleteReasoningError",
        "OntologyConflictError",
    ] {
        if message.starts_with(code) {
            return Some(code.to_owned());
        }
    }
    None
}

pub(crate) fn u32_to_usize(value: u32) -> usize {
    value as usize
}
