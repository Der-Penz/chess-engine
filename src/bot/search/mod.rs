mod move_ordering;
mod diagnostics;
pub mod pv_line;
pub mod searcher;
pub mod transposition_table;
use std::sync::{atomic::AtomicBool, Arc};

pub(super) type AbortFlag = Arc<AtomicBool>;
