//! [`vectorcraft_text::FontDb::global`] is process-wide. Tests that add fonts or look up installed
//! ones by name (`FontDb::face` loads them) must hold [`changing_fonts`] so they do not bump
//! generation while renderer frame-retention tests run in other crates.

use std::sync::{Mutex, MutexGuard, PoisonError};

pub(crate) fn changing_fonts() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(PoisonError::into_inner)
}
