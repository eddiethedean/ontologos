//! Typed errors for Node.js bindings.

use napi::bindgen_prelude::*;
use napi_derive::napi;

pub(crate) fn map_err(error: ontologos_js::JsError) -> Error {
    let message = error.to_string();
    match error.code() {
        "ParseError" => Error::new(Status::InvalidArg, message),
        "ResourceLimitError" => Error::new(Status::GenericFailure, message),
        "IncompleteReasoningError" => Error::new(Status::GenericFailure, message),
        _ => Error::new(Status::GenericFailure, message),
    }
}

/// Returns the OntoLogos error code prefix from a Node error message, if present.
#[napi]
pub fn error_code_from_message(message: String) -> Option<String> {
    for code in [
        "ParseError",
        "ResourceLimitError",
        "IncompleteReasoningError",
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
