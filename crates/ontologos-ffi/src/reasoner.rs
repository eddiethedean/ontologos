//! C FFI bindings for [`JsReasoner`].

use std::os::raw::{c_char, c_int, c_longlong};

use ontologos_js::JsReasoner;
use serde_json::Value;

use crate::error::{clear_error, set_error, set_message_error};
use crate::handles::{drop_reasoner_handle, into_reasoner_handle, with_ontology, with_reasoner};
use crate::strings::{optional_u64, read_cstr, read_required_cstr, return_string};

fn load_reasoner(build: impl FnOnce() -> ontologos_js::Result<JsReasoner>) -> c_longlong {
    clear_error();
    match build() {
        Ok(reasoner) => into_reasoner_handle(reasoner),
        Err(error) => {
            set_error(error);
            0
        }
    }
}

fn with_reasoner_op<F, R>(handle: c_longlong, f: F) -> Result<R, ()>
where
    F: FnOnce(&mut JsReasoner) -> ontologos_js::Result<R>,
{
    clear_error();
    if handle == 0 {
        set_message_error("invalid reasoner handle");
        return Err(());
    }
    match with_reasoner(handle, f) {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(error)) => {
            set_error(error);
            Err(())
        }
        Err(()) => {
            set_message_error("invalid or stale reasoner handle");
            Err(())
        }
    }
}

fn json_result(
    handle: c_longlong,
    f: impl FnOnce(&mut JsReasoner) -> ontologos_js::Result<Value>,
) -> *mut c_char {
    match with_reasoner_op(handle, f) {
        Ok(value) => return_string(value.to_string()),
        Err(()) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_reasoner_new(
    ontology_handle: c_longlong,
    profile: *const c_char,
    incremental: c_int,
    budget_secs: c_longlong,
) -> c_longlong {
    if ontology_handle == 0 {
        set_message_error("invalid ontology handle");
        return 0;
    }
    let profile = unsafe { read_cstr(profile) };
    match with_ontology(ontology_handle, |ontology| {
        JsReasoner::from_ontology(
            ontology,
            profile.as_deref(),
            incremental != 0,
            optional_u64(budget_secs),
        )
    }) {
        Ok(Ok(reasoner)) => into_reasoner_handle(reasoner),
        Ok(Err(error)) => {
            set_error(error);
            0
        }
        Err(()) => {
            set_message_error("invalid or stale ontology handle");
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_reasoner_from_path(
    path: *const c_char,
    profile: *const c_char,
    incremental: c_int,
    budget_secs: c_longlong,
    lenient: c_int,
    trusted: c_int,
) -> c_longlong {
    let Some(path) = (unsafe { read_required_cstr(path, "path") }) else {
        return 0;
    };
    if trusted == 0 {
        set_message_error(
            "unsandboxed path load rejected; pass trusted=1 for local trusted paths or use ontologos_reasoner_load_in",
        );
        return 0;
    }
    let profile = unsafe { read_cstr(profile) };
    load_reasoner(|| {
        JsReasoner::from_path(
            &path,
            profile.as_deref(),
            incremental != 0,
            optional_u64(budget_secs),
            lenient != 0,
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_reasoner_load_in(
    base: *const c_char,
    path: *const c_char,
    profile: *const c_char,
    incremental: c_int,
    budget_secs: c_longlong,
    lenient: c_int,
) -> c_longlong {
    let Some(base) = (unsafe { read_required_cstr(base, "base") }) else {
        return 0;
    };
    let Some(path) = (unsafe { read_required_cstr(path, "path") }) else {
        return 0;
    };
    let profile = unsafe { read_cstr(profile) };
    load_reasoner(|| {
        JsReasoner::load_in(
            &base,
            &path,
            profile.as_deref(),
            incremental != 0,
            optional_u64(budget_secs),
            lenient != 0,
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_reasoner_parse_meta(handle: c_longlong) -> *mut c_char {
    json_result(handle, |reasoner| reasoner.parse_meta())
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_reasoner_taxonomy(handle: c_longlong) -> *mut c_char {
    match with_reasoner_op(handle, |reasoner| reasoner.taxonomy()) {
        Ok(Some(value)) => return_string(value.to_string()),
        Ok(None) => std::ptr::null_mut(),
        Err(()) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_reasoner_classify(handle: c_longlong) -> *mut c_char {
    json_result(handle, |reasoner| reasoner.classify())
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_reasoner_explain(handle: c_longlong) -> *mut c_char {
    json_result(handle, |reasoner| reasoner.explain())
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_reasoner_check_consistency(handle: c_longlong) -> *mut c_char {
    json_result(handle, |reasoner| reasoner.check_consistency())
}

/// Returns 1 if consistent, 0 if inconsistent, -1 on error.
#[unsafe(no_mangle)]
pub extern "C" fn ontologos_reasoner_is_consistent(handle: c_longlong) -> c_int {
    match with_reasoner_op(handle, |reasoner| reasoner.is_consistent()) {
        Ok(value) => i32::from(value),
        Err(()) => -1,
    }
}

/// Returns 1 if entailed, 0 if not entailed, -1 on error.
#[unsafe(no_mangle)]
pub extern "C" fn ontologos_reasoner_is_entailed(
    handle: c_longlong,
    sub: *const c_char,
    sup: *const c_char,
    individual: *const c_char,
    class_iri: *const c_char,
    subject: *const c_char,
    property: *const c_char,
    object: *const c_char,
) -> c_int {
    let sub = unsafe { read_cstr(sub) };
    let sup = unsafe { read_cstr(sup) };
    let individual = unsafe { read_cstr(individual) };
    let class_iri = unsafe { read_cstr(class_iri) };
    let subject = unsafe { read_cstr(subject) };
    let property = unsafe { read_cstr(property) };
    let object = unsafe { read_cstr(object) };
    match with_reasoner_op(handle, |reasoner| {
        reasoner.is_entailed(
            sub.as_deref(),
            sup.as_deref(),
            individual.as_deref(),
            class_iri.as_deref(),
            subject.as_deref(),
            property.as_deref(),
            object.as_deref(),
        )
    }) {
        Ok(value) => i32::from(value),
        Err(()) => -1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_reasoner_query(
    handle: c_longlong,
    query: *const c_char,
) -> *mut c_char {
    let Some(query) = (unsafe { read_required_cstr(query, "query") }) else {
        return std::ptr::null_mut();
    };
    json_result(handle, |reasoner| reasoner.query(&query))
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_reasoner_add_subclass_of(
    handle: c_longlong,
    subclass: *const c_char,
    superclass: *const c_char,
) -> c_longlong {
    let Some(subclass) = (unsafe { read_required_cstr(subclass, "subclass") }) else {
        return handle;
    };
    let Some(superclass) = (unsafe { read_required_cstr(superclass, "superclass") }) else {
        return handle;
    };
    match with_reasoner_op(handle, |reasoner| {
        reasoner.add_subclass_of(&subclass, &superclass)
    }) {
        Ok(()) => handle,
        Err(()) => handle,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_reasoner_remove_subclass_of(
    handle: c_longlong,
    subclass: *const c_char,
    superclass: *const c_char,
) -> c_longlong {
    let Some(subclass) = (unsafe { read_required_cstr(subclass, "subclass") }) else {
        return handle;
    };
    let Some(superclass) = (unsafe { read_required_cstr(superclass, "superclass") }) else {
        return handle;
    };
    match with_reasoner_op(handle, |reasoner| {
        reasoner.remove_subclass_of(&subclass, &superclass)
    }) {
        Ok(()) => handle,
        Err(()) => handle,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_reasoner_add_axiom_json(
    handle: c_longlong,
    axiom_json: *const c_char,
) -> c_longlong {
    let Some(axiom_json) = (unsafe { read_required_cstr(axiom_json, "axiom") }) else {
        return handle;
    };
    let axiom: Value = match serde_json::from_str(&axiom_json) {
        Ok(value) => value,
        Err(error) => {
            set_error(ontologos_js::JsError::Parse(error.to_string()));
            return handle;
        }
    };
    match with_reasoner_op(handle, |reasoner| reasoner.add_axiom_json(&axiom)) {
        Ok(()) => handle,
        Err(()) => handle,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_reasoner_close(handle: c_longlong) {
    if handle != 0 {
        let _ = drop_reasoner_handle(handle);
    }
}
