mod diagnostics;
mod move_ordering;
mod opening_book;
mod pv_line;
pub mod repetition_history;
pub mod searcher;
pub mod transposition_table;
use std::sync::{atomic::AtomicBool, Arc};

pub(super) type AbortFlag = Arc<AtomicBool>;
