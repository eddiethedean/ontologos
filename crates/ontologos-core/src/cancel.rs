//! Cooperative cancellation for long-running DL/tableau work.

use std::cell::RefCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

thread_local! {
    static CURRENT_CANCEL: RefCell<Option<Arc<AtomicBool>>> = const { RefCell::new(None) };
}

/// Install a cancel flag for the current worker thread.
pub fn set_current_cancel(flag: Option<Arc<AtomicBool>>) {
    CURRENT_CANCEL.with(|slot| *slot.borrow_mut() = flag);
}

/// Whether the current worker should stop (budget timeout).
#[must_use]
pub fn cancel_requested() -> bool {
    CURRENT_CANCEL.with(|slot| {
        slot.borrow()
            .as_ref()
            .is_some_and(|flag| flag.load(Ordering::Acquire))
    })
}
