//! Opaque handle helpers for C pointer passing.

/// Encode a heap value as an opaque handle (`i64`).
pub fn into_handle<T>(value: T) -> i64 {
    Box::into_raw(Box::new(value)) as i64
}

/// Borrow a heap value from an opaque handle.
///
/// # Safety
///
/// `handle` must be a valid pointer previously returned by [`into_handle`].
pub unsafe fn borrow_handle<T>(handle: i64) -> &'static mut T {
    unsafe { &mut *(handle as *mut T) }
}

/// Drop a heap value referenced by an opaque handle.
///
/// # Safety
///
/// `handle` must be a valid, un-dropped pointer previously returned by [`into_handle`].
pub unsafe fn drop_handle<T>(handle: i64) {
    if handle != 0 {
        unsafe {
            drop(Box::from_raw(handle as *mut T));
        }
    }
}
