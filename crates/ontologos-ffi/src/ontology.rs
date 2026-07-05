//! C FFI bindings for [`JsOntology`].

use std::os::raw::{c_char, c_int, c_longlong};

use ontologos_js::JsOntology;

use crate::error::{clear_error, set_error, set_message_error};
use crate::handles::{drop_ontology_handle, into_ontology_handle, with_ontology};
use crate::strings::{optional_usize, read_required_cstr, return_string};

const DEFAULT_MAX_BYTES: usize = 64 * 1024 * 1024;

fn load_ontology(build: impl FnOnce() -> ontologos_js::Result<JsOntology>) -> c_longlong {
    clear_error();
    match build() {
        Ok(ontology) => into_ontology_handle(ontology),
        Err(error) => {
            set_error(error);
            0
        }
    }
}

fn validate_byte_len(len: usize) -> Result<(), ()> {
    if len > DEFAULT_MAX_BYTES {
        set_message_error(format!(
            "byte length {len} exceeds limit of {DEFAULT_MAX_BYTES} bytes"
        ));
        Err(())
    } else {
        Ok(())
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_ontology_from_json(json: *const c_char) -> c_longlong {
    let Some(json) = (unsafe { read_required_cstr(json, "json") }) else {
        return 0;
    };
    load_ontology(|| JsOntology::from_json(&json))
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_ontology_from_json_with_limits(
    json: *const c_char,
    max_json_bytes: c_longlong,
    max_entities: c_longlong,
    max_axioms: c_longlong,
    max_iri_len: c_longlong,
) -> c_longlong {
    let Some(json) = (unsafe { read_required_cstr(json, "json") }) else {
        return 0;
    };
    load_ontology(|| {
        JsOntology::from_json_with_limits(
            &json,
            optional_usize(max_json_bytes),
            optional_usize(max_entities),
            optional_usize(max_axioms),
            optional_usize(max_iri_len),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_ontology_from_bytes(data: *const u8, len: usize) -> c_longlong {
    clear_error();
    if data.is_null() && len > 0 {
        set_message_error("null bytes argument");
        return 0;
    }
    if validate_byte_len(len).is_err() {
        return 0;
    }
    let bytes = if len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(data, len) }
    };
    load_ontology(|| JsOntology::load_bytes(bytes))
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_ontology_from_bytes_lenient(data: *const u8, len: usize) -> c_longlong {
    clear_error();
    if data.is_null() && len > 0 {
        set_message_error("null bytes argument");
        return 0;
    }
    if validate_byte_len(len).is_err() {
        return 0;
    }
    let bytes = if len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(data, len) }
    };
    load_ontology(|| JsOntology::load_bytes_lenient(bytes))
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_ontology_from_text(text: *const c_char) -> c_longlong {
    let Some(text) = (unsafe { read_required_cstr(text, "text") }) else {
        return 0;
    };
    load_ontology(|| JsOntology::load_text(&text))
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_ontology_from_text_lenient(text: *const c_char) -> c_longlong {
    let Some(text) = (unsafe { read_required_cstr(text, "text") }) else {
        return 0;
    };
    load_ontology(|| JsOntology::load_text_lenient(&text))
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_ontology_load(
    path: *const c_char,
    lenient: c_int,
    trusted: c_int,
) -> c_longlong {
    let Some(path) = (unsafe { read_required_cstr(path, "path") }) else {
        return 0;
    };
    if trusted == 0 {
        set_message_error(
            "unsandboxed path load rejected; pass trusted=1 for local trusted paths or use ontologos_ontology_load_in",
        );
        return 0;
    }
    load_ontology(|| JsOntology::load_path(&path, lenient != 0))
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_ontology_load_in(
    base: *const c_char,
    path: *const c_char,
    lenient: c_int,
) -> c_longlong {
    let Some(base) = (unsafe { read_required_cstr(base, "base") }) else {
        return 0;
    };
    let Some(path) = (unsafe { read_required_cstr(path, "path") }) else {
        return 0;
    };
    load_ontology(|| JsOntology::load_in(&base, &path, lenient != 0))
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_ontology_to_json(handle: c_longlong) -> *mut c_char {
    clear_error();
    if handle == 0 {
        set_message_error("invalid ontology handle");
        return std::ptr::null_mut();
    }
    match with_ontology(handle, |ontology| ontology.to_json()) {
        Ok(Ok(json)) => return_string(json),
        Ok(Err(error)) => {
            set_error(error);
            std::ptr::null_mut()
        }
        Err(()) => {
            set_message_error("invalid or stale ontology handle");
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_ontology_axiom_count(handle: c_longlong) -> c_longlong {
    clear_error();
    if handle == 0 {
        set_message_error("invalid ontology handle");
        return -1;
    }
    match with_ontology(handle, |ontology| ontology.axiom_count()) {
        Ok(Ok(count)) => count as c_longlong,
        Ok(Err(error)) => {
            set_error(error);
            -1
        }
        Err(()) => {
            set_message_error("invalid or stale ontology handle");
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_ontology_entity_count(handle: c_longlong) -> c_longlong {
    clear_error();
    if handle == 0 {
        set_message_error("invalid ontology handle");
        return -1;
    }
    match with_ontology(handle, |ontology| ontology.entity_count()) {
        Ok(Ok(count)) => count as c_longlong,
        Ok(Err(error)) => {
            set_error(error);
            -1
        }
        Err(()) => {
            set_message_error("invalid or stale ontology handle");
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_ontology_close(handle: c_longlong) {
    if handle != 0 {
        let _ = drop_ontology_handle(handle);
    }
}
