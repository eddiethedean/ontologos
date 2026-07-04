//! JNI bindings for [`JsReasoner`].

use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jlong, jstring};
use ontologos_js::{JsOntology, JsReasoner};
use serde_json::Value;

use crate::error::{java_string, optional_u64, read_string, throw_error, throw_message};
use crate::handles::{borrow_handle, drop_handle, into_handle};

fn load_reasoner(
    env: &mut JNIEnv,
    _err_class: JClass<'_>,
    build: impl FnOnce() -> ontologos_js::Result<JsReasoner>,
) -> jlong {
    match build() {
        Ok(reasoner) => into_handle(reasoner),
        Err(error) => {
            let _ = throw_error(env, error);
            0
        }
    }
}

fn with_reasoner<F, R>(env: &mut JNIEnv, class: JClass<'_>, handle: jlong, f: F) -> Result<R, ()>
where
    F: FnOnce(&mut JsReasoner) -> ontologos_js::Result<R>,
{
    if handle == 0 {
        let _ = throw_message(env, class, "invalid reasoner handle");
        return Err(());
    }
    let reasoner = unsafe { borrow_handle::<JsReasoner>(handle) };
    match f(reasoner) {
        Ok(value) => Ok(value),
        Err(error) => {
            let _ = throw_error(env, error);
            Err(())
        }
    }
}

fn json_result(
    env: &mut JNIEnv,
    class: JClass<'_>,
    handle: jlong,
    f: impl FnOnce(&mut JsReasoner) -> ontologos_js::Result<Value>,
) -> jstring {
    match with_reasoner(env, class, handle, f) {
        Ok(value) => {
            let json = value.to_string();
            java_string(env, &json).unwrap_or(std::ptr::null_mut())
        }
        Err(()) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Reasoner_nativeNew(
    mut env: JNIEnv,
    class: JClass,
    ontology_handle: jlong,
    profile: JString,
    incremental: jboolean,
    budget_secs: jlong,
) -> jlong {
    if ontology_handle == 0 {
        return throw_message(&mut env, class, "invalid ontology handle");
    }
    let profile = optional_string(&mut env, profile);
    let ontology = unsafe { borrow_handle::<JsOntology>(ontology_handle) };
    load_reasoner(&mut env, class, || {
        JsReasoner::from_ontology(
            ontology,
            profile.as_deref(),
            incremental != 0,
            optional_u64(budget_secs),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Reasoner_nativeFromPath(
    mut env: JNIEnv,
    class: JClass,
    path: JString,
    profile: JString,
    incremental: jboolean,
    budget_secs: jlong,
    lenient: jboolean,
) -> jlong {
    let Ok(path) = read_string(&mut env, &path) else {
        return throw_message(&mut env, class, "invalid UTF-8 in path argument");
    };
    let profile = optional_string(&mut env, profile);
    load_reasoner(&mut env, class, || {
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
pub extern "system" fn Java_dev_ontologos_Reasoner_nativeLoadIn(
    mut env: JNIEnv,
    class: JClass,
    base: JString,
    path: JString,
    profile: JString,
    incremental: jboolean,
    budget_secs: jlong,
    lenient: jboolean,
) -> jlong {
    let Ok(base) = read_string(&mut env, &base) else {
        return throw_message(&mut env, class, "invalid UTF-8 in base argument");
    };
    let Ok(path) = read_string(&mut env, &path) else {
        return throw_message(&mut env, class, "invalid UTF-8 in path argument");
    };
    let profile = optional_string(&mut env, profile);
    load_reasoner(&mut env, class, || {
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
pub extern "system" fn Java_dev_ontologos_Reasoner_nativeParseMeta(
    mut env: JNIEnv,
    class: JClass,
    handle: jlong,
) -> jstring {
    json_result(&mut env, class, handle, |reasoner| reasoner.parse_meta())
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Reasoner_nativeTaxonomy(
    mut env: JNIEnv,
    class: JClass,
    handle: jlong,
) -> jstring {
    match with_reasoner(&mut env, class, handle, |reasoner| reasoner.taxonomy()) {
        Ok(Some(value)) => {
            let json = value.to_string();
            java_string(&mut env, &json).unwrap_or(std::ptr::null_mut())
        }
        Ok(None) => std::ptr::null_mut(),
        Err(()) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Reasoner_nativeClassify(
    mut env: JNIEnv,
    class: JClass,
    handle: jlong,
) -> jstring {
    if handle == 0 {
        let _ = throw_message(&mut env, class, "invalid reasoner handle");
        return std::ptr::null_mut();
    }
    let reasoner = unsafe { borrow_handle::<JsReasoner>(handle) };
    match reasoner.classify() {
        Ok(value) => java_string(&mut env, &value.to_string()).unwrap_or(std::ptr::null_mut()),
        Err(error) => {
            let _ = throw_error(&mut env, error);
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Reasoner_nativeExplain(
    mut env: JNIEnv,
    class: JClass,
    handle: jlong,
) -> jstring {
    json_result(&mut env, class, handle, |reasoner| reasoner.explain())
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Reasoner_nativeCheckConsistency(
    mut env: JNIEnv,
    class: JClass,
    handle: jlong,
) -> jstring {
    json_result(&mut env, class, handle, |reasoner| {
        reasoner.check_consistency()
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Reasoner_nativeIsConsistent(
    mut env: JNIEnv,
    class: JClass,
    handle: jlong,
) -> jboolean {
    match with_reasoner(&mut env, class, handle, |reasoner| reasoner.is_consistent()) {
        Ok(value) => i32::from(value) as jboolean,
        Err(()) => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Reasoner_nativeIsEntailed(
    mut env: JNIEnv,
    class: JClass,
    handle: jlong,
    sub: JString,
    sup: JString,
    individual: JString,
    class_iri: JString,
    subject: JString,
    property: JString,
    object: JString,
) -> jboolean {
    let sub = optional_string(&mut env, sub);
    let sup = optional_string(&mut env, sup);
    let individual = optional_string(&mut env, individual);
    let class_iri = optional_string(&mut env, class_iri);
    let subject = optional_string(&mut env, subject);
    let property = optional_string(&mut env, property);
    let object = optional_string(&mut env, object);
    match with_reasoner(&mut env, class, handle, |reasoner| {
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
        Ok(value) => i32::from(value) as jboolean,
        Err(()) => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Reasoner_nativeQuery(
    mut env: JNIEnv,
    class: JClass,
    handle: jlong,
    query: JString,
) -> jstring {
    let Ok(query) = read_string(&mut env, &query) else {
        let _ = throw_message(&mut env, class, "invalid UTF-8 in query argument");
        return std::ptr::null_mut();
    };
    json_result(&mut env, class, handle, |reasoner| reasoner.query(&query))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Reasoner_nativeAddSubclassOf(
    mut env: JNIEnv,
    class: JClass,
    handle: jlong,
    subclass: JString,
    superclass: JString,
) -> jlong {
    let Ok(subclass) = read_string(&mut env, &subclass) else {
        return throw_message(&mut env, class, "invalid UTF-8 in subclass argument");
    };
    let Ok(superclass) = read_string(&mut env, &superclass) else {
        return throw_message(&mut env, class, "invalid UTF-8 in superclass argument");
    };
    match with_reasoner(&mut env, class, handle, |reasoner| {
        reasoner.add_subclass_of(&subclass, &superclass)
    }) {
        Ok(()) => handle,
        Err(()) => handle,
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Reasoner_nativeRemoveSubclassOf(
    mut env: JNIEnv,
    class: JClass,
    handle: jlong,
    subclass: JString,
    superclass: JString,
) -> jlong {
    let Ok(subclass) = read_string(&mut env, &subclass) else {
        return throw_message(&mut env, class, "invalid UTF-8 in subclass argument");
    };
    let Ok(superclass) = read_string(&mut env, &superclass) else {
        return throw_message(&mut env, class, "invalid UTF-8 in superclass argument");
    };
    match with_reasoner(&mut env, class, handle, |reasoner| {
        reasoner.remove_subclass_of(&subclass, &superclass)
    }) {
        Ok(()) => handle,
        Err(()) => handle,
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Reasoner_nativeAddAxiomJson(
    mut env: JNIEnv,
    class: JClass,
    handle: jlong,
    axiom_json: JString,
) -> jlong {
    let Ok(axiom_json) = read_string(&mut env, &axiom_json) else {
        return throw_message(&mut env, class, "invalid UTF-8 in axiom argument");
    };
    let axiom: Value = match serde_json::from_str(&axiom_json) {
        Ok(value) => value,
        Err(error) => {
            let _ = throw_error(&mut env, ontologos_js::JsError::Parse(error.to_string()));
            return handle;
        }
    };
    match with_reasoner(&mut env, class, handle, |reasoner| {
        reasoner.add_axiom_json(&axiom)
    }) {
        Ok(()) => handle,
        Err(()) => handle,
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Reasoner_nativeClose(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    unsafe {
        drop_handle::<JsReasoner>(handle);
    }
}

fn optional_string(env: &mut JNIEnv, value: JString) -> Option<String> {
    if value.is_null() {
        None
    } else {
        read_string(env, &value).ok()
    }
}
