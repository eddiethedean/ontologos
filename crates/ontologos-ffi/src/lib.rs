//! Shared C ABI for OntoLogos language bindings.

#![allow(unsafe_code)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

mod builder;
mod error;
mod handles;
mod ontology;
mod reasoner;
mod strings;

use std::os::raw::c_char;

use ontologos_js::VERSION;

use crate::strings::{read_required_cstr, return_string};

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_version() -> *mut c_char {
    return_string(VERSION.to_string())
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_error_code_from_message(message: *const c_char) -> *mut c_char {
    let Some(message) = (unsafe { read_required_cstr(message, "message") }) else {
        return std::ptr::null_mut();
    };
    for code in [
        "ParseError",
        "ResourceLimitError",
        "IncompleteReasoningError",
        "OntologyConflictError",
    ] {
        if message.starts_with(code) {
            return return_string(code.to_string());
        }
    }
    std::ptr::null_mut()
}
