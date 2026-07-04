//! Map shared binding errors to Java exceptions.

use jni::JNIEnv;
use jni::objects::JClass;
use ontologos_js::JsError;

pub fn throw_error(env: &mut JNIEnv, error: JsError) -> jni::errors::Result<()> {
    let (class_name, message) = match &error {
        JsError::Parse(msg) => ("dev/ontologos/ParseException", msg.clone()),
        JsError::ResourceLimit(msg) => ("dev/ontologos/ResourceLimitException", msg.clone()),
        JsError::IncompleteReasoning => (
            "dev/ontologos/IncompleteReasoningException",
            error.to_string(),
        ),
        JsError::OntologyConflict => ("dev/ontologos/OntologyConflictException", error.to_string()),
        JsError::Other(msg) => ("dev/ontologos/OntologosException", msg.clone()),
    };
    let class = env.find_class(class_name)?;
    env.throw_new(class, message)?;
    Ok(())
}

pub fn throw_message(env: &mut JNIEnv, class: JClass<'_>, message: impl AsRef<str>) -> i64 {
    let _ = env.throw_new(class, message.as_ref());
    0
}

pub fn java_string(env: &mut JNIEnv, value: &str) -> jni::errors::Result<jni::sys::jstring> {
    env.new_string(value).map(|s| s.into_raw())
}

pub fn read_string(env: &mut JNIEnv, value: &jni::objects::JString) -> jni::errors::Result<String> {
    env.get_string(value)
        .map(|s| s.to_string_lossy().into_owned())
}

pub fn optional_usize(value: i64) -> Option<usize> {
    if value < 0 {
        None
    } else {
        Some(value as usize)
    }
}

pub fn optional_u64(value: i64) -> Option<u64> {
    if value < 0 { None } else { Some(value as u64) }
}
