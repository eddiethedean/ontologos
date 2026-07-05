//! C FFI bindings for [`JsOntologyBuilder`].

use std::os::raw::{c_char, c_longlong};

use ontologos_js::JsOntologyBuilder;

use crate::error::{clear_error, set_error, set_message_error};
use crate::handles::{
    drop_builder_handle, into_builder_handle, into_ontology_handle, with_builder,
};
use crate::strings::read_required_cstr;

fn with_builder_op<F>(handle: c_longlong, f: F) -> c_longlong
where
    F: FnOnce(&mut JsOntologyBuilder) -> ontologos_js::Result<()>,
{
    clear_error();
    if handle == 0 {
        set_message_error("invalid builder handle");
        return 0;
    }
    match with_builder(handle, f) {
        Ok(Ok(())) => handle,
        Ok(Err(error)) => {
            set_error(error);
            handle
        }
        Err(()) => {
            set_message_error("invalid or stale builder handle");
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_builder_new() -> c_longlong {
    clear_error();
    into_builder_handle(JsOntologyBuilder::new())
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_builder_add_class(
    handle: c_longlong,
    iri: *const c_char,
) -> c_longlong {
    let Some(iri) = (unsafe { read_required_cstr(iri, "iri") }) else {
        return handle;
    };
    with_builder_op(handle, |builder| builder.add_class(&iri))
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_builder_individual(
    handle: c_longlong,
    iri: *const c_char,
) -> c_longlong {
    let Some(iri) = (unsafe { read_required_cstr(iri, "iri") }) else {
        return handle;
    };
    with_builder_op(handle, |builder| builder.individual(&iri))
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_builder_object_property(
    handle: c_longlong,
    iri: *const c_char,
) -> c_longlong {
    let Some(iri) = (unsafe { read_required_cstr(iri, "iri") }) else {
        return handle;
    };
    with_builder_op(handle, |builder| builder.object_property(&iri))
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_builder_subclass_of(
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
    with_builder_op(handle, |builder| {
        builder.subclass_of(&subclass, &superclass)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_builder_subproperty_of(
    handle: c_longlong,
    sub: *const c_char,
    sup: *const c_char,
) -> c_longlong {
    let Some(sub) = (unsafe { read_required_cstr(sub, "sub") }) else {
        return handle;
    };
    let Some(sup) = (unsafe { read_required_cstr(sup, "sup") }) else {
        return handle;
    };
    with_builder_op(handle, |builder| builder.subproperty_of(&sub, &sup))
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_builder_property_domain(
    handle: c_longlong,
    property: *const c_char,
    domain: *const c_char,
) -> c_longlong {
    let Some(property) = (unsafe { read_required_cstr(property, "property") }) else {
        return handle;
    };
    let Some(domain) = (unsafe { read_required_cstr(domain, "domain") }) else {
        return handle;
    };
    with_builder_op(handle, |builder| {
        builder.property_domain(&property, &domain)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_builder_property_range(
    handle: c_longlong,
    property: *const c_char,
    range: *const c_char,
) -> c_longlong {
    let Some(property) = (unsafe { read_required_cstr(property, "property") }) else {
        return handle;
    };
    let Some(range) = (unsafe { read_required_cstr(range, "range") }) else {
        return handle;
    };
    with_builder_op(handle, |builder| builder.property_range(&property, &range))
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_builder_class_assertion(
    handle: c_longlong,
    individual: *const c_char,
    class_iri: *const c_char,
) -> c_longlong {
    let Some(individual) = (unsafe { read_required_cstr(individual, "individual") }) else {
        return handle;
    };
    let Some(class_iri) = (unsafe { read_required_cstr(class_iri, "class") }) else {
        return handle;
    };
    with_builder_op(handle, |builder| {
        builder.class_assertion(&individual, &class_iri)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_builder_object_property_assertion(
    handle: c_longlong,
    subject: *const c_char,
    property: *const c_char,
    object: *const c_char,
) -> c_longlong {
    let Some(subject) = (unsafe { read_required_cstr(subject, "subject") }) else {
        return handle;
    };
    let Some(property) = (unsafe { read_required_cstr(property, "property") }) else {
        return handle;
    };
    let Some(object) = (unsafe { read_required_cstr(object, "object") }) else {
        return handle;
    };
    with_builder_op(handle, |builder| {
        builder.object_property_assertion(&subject, &property, &object)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_builder_build(handle: c_longlong) -> c_longlong {
    clear_error();
    if handle == 0 {
        set_message_error("invalid builder handle");
        return 0;
    }
    match with_builder(handle, |builder| builder.build()) {
        Ok(Ok(ontology)) => {
            drop_builder_handle(handle);
            into_ontology_handle(ontology)
        }
        Ok(Err(error)) => {
            set_error(error);
            0
        }
        Err(()) => {
            set_message_error("invalid or stale builder handle");
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_builder_close(handle: c_longlong) {
    if handle != 0 {
        let _ = drop_builder_handle(handle);
    }
}
