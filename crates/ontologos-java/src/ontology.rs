//! JNI bindings for [`JsOntology`].

use jni::JNIEnv;
use jni::objects::{JByteArray, JClass, JString};
use jni::sys::{jlong, jstring};
use ontologos_js::JsOntology;

use crate::error::{java_string, optional_usize, read_string, throw_error, throw_message};
use crate::handles::{borrow_handle, drop_handle, into_handle};

fn load_ontology(
    env: &mut JNIEnv,
    _err_class: JClass<'_>,
    build: impl FnOnce() -> ontologos_js::Result<JsOntology>,
) -> jlong {
    match build() {
        Ok(ontology) => into_handle(ontology),
        Err(error) => {
            let _ = throw_error(env, error);
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Ontology_nativeFromJson(
    mut env: JNIEnv,
    class: JClass,
    json: JString,
) -> jlong {
    let Ok(json) = read_string(&mut env, &json) else {
        return throw_message(&mut env, class, "invalid UTF-8 in json argument");
    };
    load_ontology(&mut env, class, || JsOntology::from_json(&json))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Ontology_nativeFromJsonWithLimits(
    mut env: JNIEnv,
    class: JClass,
    json: JString,
    max_json_bytes: jlong,
    max_entities: jlong,
    max_axioms: jlong,
    max_iri_len: jlong,
) -> jlong {
    let Ok(json) = read_string(&mut env, &json) else {
        return throw_message(&mut env, class, "invalid UTF-8 in json argument");
    };
    load_ontology(&mut env, class, || {
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
pub extern "system" fn Java_dev_ontologos_Ontology_nativeFromBytes(
    mut env: JNIEnv,
    class: JClass,
    bytes: JByteArray,
) -> jlong {
    let Ok(len) = env.get_array_length(&bytes) else {
        return throw_message(&mut env, class, "failed to read byte array length");
    };
    let mut buffer = vec![0i8; len as usize];
    let Ok(_) = env.get_byte_array_region(&bytes, 0, &mut buffer) else {
        return throw_message(&mut env, class, "failed to read byte array");
    };
    let bytes: Vec<u8> = buffer.into_iter().map(|b| b as u8).collect();
    load_ontology(&mut env, class, || JsOntology::load_bytes(&bytes))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Ontology_nativeFromBytesLenient(
    mut env: JNIEnv,
    class: JClass,
    bytes: JByteArray,
) -> jlong {
    let Ok(len) = env.get_array_length(&bytes) else {
        return throw_message(&mut env, class, "failed to read byte array length");
    };
    let mut buffer = vec![0i8; len as usize];
    let Ok(_) = env.get_byte_array_region(&bytes, 0, &mut buffer) else {
        return throw_message(&mut env, class, "failed to read byte array");
    };
    let bytes: Vec<u8> = buffer.into_iter().map(|b| b as u8).collect();
    load_ontology(&mut env, class, || JsOntology::load_bytes_lenient(&bytes))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Ontology_nativeFromText(
    mut env: JNIEnv,
    class: JClass,
    text: JString,
) -> jlong {
    let Ok(text) = read_string(&mut env, &text) else {
        return throw_message(&mut env, class, "invalid UTF-8 in text argument");
    };
    load_ontology(&mut env, class, || JsOntology::load_text(&text))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Ontology_nativeFromTextLenient(
    mut env: JNIEnv,
    class: JClass,
    text: JString,
) -> jlong {
    let Ok(text) = read_string(&mut env, &text) else {
        return throw_message(&mut env, class, "invalid UTF-8 in text argument");
    };
    load_ontology(&mut env, class, || JsOntology::load_text_lenient(&text))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Ontology_nativeLoad(
    mut env: JNIEnv,
    class: JClass,
    path: JString,
    lenient: jni::sys::jboolean,
) -> jlong {
    let Ok(path) = read_string(&mut env, &path) else {
        return throw_message(&mut env, class, "invalid UTF-8 in path argument");
    };
    let lenient = lenient != 0;
    load_ontology(&mut env, class, || JsOntology::load_path(&path, lenient))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Ontology_nativeLoadIn(
    mut env: JNIEnv,
    class: JClass,
    base: JString,
    path: JString,
    lenient: jni::sys::jboolean,
) -> jlong {
    let Ok(base) = read_string(&mut env, &base) else {
        return throw_message(&mut env, class, "invalid UTF-8 in base argument");
    };
    let Ok(path) = read_string(&mut env, &path) else {
        return throw_message(&mut env, class, "invalid UTF-8 in path argument");
    };
    let lenient = lenient != 0;
    load_ontology(&mut env, class, || {
        JsOntology::load_in(&base, &path, lenient)
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Ontology_nativeToJson(
    mut env: JNIEnv,
    class: JClass,
    handle: jlong,
) -> jstring {
    if handle == 0 {
        let _ = throw_message(&mut env, class, "invalid ontology handle");
        return std::ptr::null_mut();
    }
    // SAFETY: handle comes from into_handle and is not yet closed.
    let ontology = unsafe { borrow_handle::<JsOntology>(handle) };
    match ontology.to_json() {
        Ok(json) => java_string(&mut env, &json).unwrap_or(std::ptr::null_mut()),
        Err(error) => {
            let _ = throw_error(&mut env, error);
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Ontology_nativeAxiomCount(
    mut env: JNIEnv,
    class: JClass,
    handle: jlong,
) -> jlong {
    if handle == 0 {
        let _ = throw_message(&mut env, class, "invalid ontology handle");
        return -1;
    }
    let ontology = unsafe { borrow_handle::<JsOntology>(handle) };
    match ontology.axiom_count() {
        Ok(count) => count as jlong,
        Err(error) => {
            let _ = throw_error(&mut env, error);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Ontology_nativeEntityCount(
    mut env: JNIEnv,
    class: JClass,
    handle: jlong,
) -> jlong {
    if handle == 0 {
        let _ = throw_message(&mut env, class, "invalid ontology handle");
        return -1;
    }
    let ontology = unsafe { borrow_handle::<JsOntology>(handle) };
    match ontology.entity_count() {
        Ok(count) => count as jlong,
        Err(error) => {
            let _ = throw_error(&mut env, error);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_Ontology_nativeClose(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    // SAFETY: Java close() must be called at most once per handle.
    unsafe {
        drop_handle::<JsOntology>(handle);
    }
}
