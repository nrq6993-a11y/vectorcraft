//! [`vectorcraft_text::FontDb::global`] is process-wide. Integration tests that add fonts must hold
//! [`changing_fonts`] so they do not bump generation during parallel CI.

use std::sync::{Mutex, MutexGuard, PoisonError};

pub(crate) fn changing_fonts() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(PoisonError::into_inner)
}
