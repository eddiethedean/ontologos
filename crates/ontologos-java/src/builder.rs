//! JNI bindings for [`JsOntologyBuilder`].

use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::jlong;
use ontologos_js::JsOntologyBuilder;

use crate::error::{read_string, throw_error, throw_message};
use crate::handles::{borrow_handle, drop_handle, into_handle};

fn with_builder<F>(env: &mut JNIEnv, class: JClass<'_>, handle: jlong, f: F) -> jlong
where
    F: FnOnce(&mut JsOntologyBuilder) -> ontologos_js::Result<()>,
{
    if handle == 0 {
        return throw_message(env, class, "invalid builder handle");
    }
    let builder = unsafe { borrow_handle::<JsOntologyBuilder>(handle) };
    match f(builder) {
        Ok(()) => handle,
        Err(error) => {
            let _ = throw_error(env, error);
            handle
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_OntologyBuilder_nativeNew(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    into_handle(JsOntologyBuilder::new())
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_OntologyBuilder_nativeAddClass(
    mut env: JNIEnv,
    class: JClass,
    handle: jlong,
    iri: JString,
) -> jlong {
    let Ok(iri) = read_string(&mut env, &iri) else {
        return throw_message(&mut env, class, "invalid UTF-8 in iri argument");
    };
    with_builder(&mut env, class, handle, |builder| builder.add_class(&iri))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_OntologyBuilder_nativeIndividual(
    mut env: JNIEnv,
    class: JClass,
    handle: jlong,
    iri: JString,
) -> jlong {
    let Ok(iri) = read_string(&mut env, &iri) else {
        return throw_message(&mut env, class, "invalid UTF-8 in iri argument");
    };
    with_builder(&mut env, class, handle, |builder| builder.individual(&iri))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_OntologyBuilder_nativeObjectProperty(
    mut env: JNIEnv,
    class: JClass,
    handle: jlong,
    iri: JString,
) -> jlong {
    let Ok(iri) = read_string(&mut env, &iri) else {
        return throw_message(&mut env, class, "invalid UTF-8 in iri argument");
    };
    with_builder(&mut env, class, handle, |builder| {
        builder.object_property(&iri)
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_OntologyBuilder_nativeSubclassOf(
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
    with_builder(&mut env, class, handle, |builder| {
        builder.subclass_of(&subclass, &superclass)
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_OntologyBuilder_nativeSubpropertyOf(
    mut env: JNIEnv,
    class: JClass,
    handle: jlong,
    sub: JString,
    sup: JString,
) -> jlong {
    let Ok(sub) = read_string(&mut env, &sub) else {
        return throw_message(&mut env, class, "invalid UTF-8 in sub argument");
    };
    let Ok(sup) = read_string(&mut env, &sup) else {
        return throw_message(&mut env, class, "invalid UTF-8 in sup argument");
    };
    with_builder(&mut env, class, handle, |builder| {
        builder.subproperty_of(&sub, &sup)
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_OntologyBuilder_nativePropertyDomain(
    mut env: JNIEnv,
    class: JClass,
    handle: jlong,
    property: JString,
    domain: JString,
) -> jlong {
    let Ok(property) = read_string(&mut env, &property) else {
        return throw_message(&mut env, class, "invalid UTF-8 in property argument");
    };
    let Ok(domain) = read_string(&mut env, &domain) else {
        return throw_message(&mut env, class, "invalid UTF-8 in domain argument");
    };
    with_builder(&mut env, class, handle, |builder| {
        builder.property_domain(&property, &domain)
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_OntologyBuilder_nativePropertyRange(
    mut env: JNIEnv,
    class: JClass,
    handle: jlong,
    property: JString,
    range: JString,
) -> jlong {
    let Ok(property) = read_string(&mut env, &property) else {
        return throw_message(&mut env, class, "invalid UTF-8 in property argument");
    };
    let Ok(range) = read_string(&mut env, &range) else {
        return throw_message(&mut env, class, "invalid UTF-8 in range argument");
    };
    with_builder(&mut env, class, handle, |builder| {
        builder.property_range(&property, &range)
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_OntologyBuilder_nativeClassAssertion(
    mut env: JNIEnv,
    class: JClass,
    handle: jlong,
    individual: JString,
    class_iri: JString,
) -> jlong {
    let Ok(individual) = read_string(&mut env, &individual) else {
        return throw_message(&mut env, class, "invalid UTF-8 in individual argument");
    };
    let Ok(class_iri) = read_string(&mut env, &class_iri) else {
        return throw_message(&mut env, class, "invalid UTF-8 in class argument");
    };
    with_builder(&mut env, class, handle, |builder| {
        builder.class_assertion(&individual, &class_iri)
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_OntologyBuilder_nativeObjectPropertyAssertion(
    mut env: JNIEnv,
    class: JClass,
    handle: jlong,
    subject: JString,
    property: JString,
    object: JString,
) -> jlong {
    let Ok(subject) = read_string(&mut env, &subject) else {
        return throw_message(&mut env, class, "invalid UTF-8 in subject argument");
    };
    let Ok(property) = read_string(&mut env, &property) else {
        return throw_message(&mut env, class, "invalid UTF-8 in property argument");
    };
    let Ok(object) = read_string(&mut env, &object) else {
        return throw_message(&mut env, class, "invalid UTF-8 in object argument");
    };
    with_builder(&mut env, class, handle, |builder| {
        builder.object_property_assertion(&subject, &property, &object)
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_OntologyBuilder_nativeBuild(
    mut env: JNIEnv,
    class: JClass,
    handle: jlong,
) -> jlong {
    if handle == 0 {
        return throw_message(&mut env, class, "invalid builder handle");
    }
    let builder = unsafe { borrow_handle::<JsOntologyBuilder>(handle) };
    match builder.build() {
        Ok(ontology) => {
            unsafe {
                drop_handle::<JsOntologyBuilder>(handle);
            }
            into_handle(ontology)
        }
        Err(error) => {
            let _ = throw_error(&mut env, error);
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_ontologos_OntologyBuilder_nativeClose(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    unsafe {
        drop_handle::<JsOntologyBuilder>(handle);
    }
}
