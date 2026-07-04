//! Map shared binding errors to thread-local C strings for FFI hosts.

use std::cell::RefCell;
use std::ffi::CString;
use std::os::raw::c_char;

use ontologos_js::JsError;

thread_local! {
    static LAST_ERROR: RefCell<Option<(CString, CString)>> = const { RefCell::new(None) };
}

pub fn clear_error() {
    LAST_ERROR.with(|slot| *slot.borrow_mut() = None);
}

pub fn set_error(error: JsError) {
    let code = CString::new(error.code()).unwrap_or_default();
    let message = CString::new(error.to_string()).unwrap_or_default();
    LAST_ERROR.with(|slot| *slot.borrow_mut() = Some((code, message)));
}

pub fn set_message_error(message: impl Into<String>) {
    set_error(JsError::Other(message.into()));
}

fn last_field(index: usize) -> *const c_char {
    LAST_ERROR.with(|slot| {
        slot.borrow()
            .as_ref()
            .map_or(std::ptr::null(), |(code, message)| {
                if index == 0 {
                    code.as_ptr()
                } else {
                    message.as_ptr()
                }
            })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_last_error_code() -> *const c_char {
    last_field(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_last_error_message() -> *const c_char {
    last_field(1)
}

#[unsafe(no_mangle)]
pub extern "C" fn ontologos_clear_last_error() {
    clear_error();
}
