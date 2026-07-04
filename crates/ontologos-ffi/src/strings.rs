//! UTF-8 C string helpers.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::error::set_message_error;

/// Read a NUL-terminated UTF-8 string from the caller.
///
/// # Safety
///
/// `value` must be null or a valid NUL-terminated UTF-8 pointer for the call duration.
pub unsafe fn read_cstr(value: *const c_char) -> Option<String> {
    if value.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(value) }
        .to_str()
        .ok()
        .map(str::to_owned)
}

/// Read a required NUL-terminated UTF-8 string; sets error and returns `None` on failure.
///
/// # Safety
///
/// Same as [`read_cstr`].
pub unsafe fn read_required_cstr(value: *const c_char, arg: &str) -> Option<String> {
    if value.is_null() {
        set_message_error(format!("null {arg} argument"));
        return None;
    }
    match unsafe { CStr::from_ptr(value).to_str() } {
        Ok(text) => Some(text.to_owned()),
        Err(_) => {
            set_message_error(format!("invalid UTF-8 in {arg} argument"));
            None
        }
    }
}

/// Return an owned C string for the caller (must free with [`ontologos_string_free`]).
pub fn return_string(value: String) -> *mut c_char {
    CString::new(value)
        .map(CString::into_raw)
        .unwrap_or(std::ptr::null_mut())
}

/// Free a string previously returned by this library.
///
/// # Safety
///
/// `value` must be null or a pointer returned by [`return_string`] and not yet freed.
#[unsafe(no_mangle)]
pub extern "C" fn ontologos_string_free(value: *mut c_char) {
    if !value.is_null() {
        unsafe {
            drop(CString::from_raw(value));
        }
    }
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
