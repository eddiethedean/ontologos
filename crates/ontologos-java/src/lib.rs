//! JNI bindings for OntoLogos Java API.

#![allow(unsafe_code)]
#![allow(clippy::too_many_arguments)]

mod builder;
mod error;
mod handles;
mod ontology;
mod reasoner;

use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::jstring;
use ontologos_js::VERSION;

use crate::error::java_string;

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Ontologos_nativeVersion(
    mut env: JNIEnv,
    _class: JClass,
) -> jstring {
    java_string(&mut env, VERSION).unwrap_or(std::ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Ontologos_nativeErrorCodeFromMessage(
    mut env: JNIEnv,
    _class: JClass,
    message: jni::objects::JString,
) -> jstring {
    let Ok(message) = crate::error::read_string(&mut env, &message) else {
        return std::ptr::null_mut();
    };
    for code in [
        "ParseError",
        "ResourceLimitError",
        "IncompleteReasoningError",
        "OntologyConflictError",
    ] {
        if message.starts_with(code) {
            return java_string(&mut env, code).unwrap_or(std::ptr::null_mut());
        }
    }
    std::ptr::null_mut()
}
