mod diagnostics;
pub mod limit;
mod move_ordering;
mod opening_book;
mod pv_line;
pub mod repetition_history;
pub mod searcher;
pub mod transposition_table;
pub use opening_book::DEFAULT_OPENING_BOOK_ENABLED;
use std::sync::{atomic::AtomicBool, Arc};

pub(super) type AbortFlag = Arc<AtomicBool>;
