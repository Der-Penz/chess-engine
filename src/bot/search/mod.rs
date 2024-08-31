mod diagnostics;
use std::sync::{atomic::AtomicBool, mpsc::Sender, Arc};

use crate::game::{Board, Move};

use super::{evaluation::eval::Eval, ReactionMessage};

pub mod min_max;

pub type AbortFlag = Arc<AtomicBool>;

pub trait Search {
    fn set_communication_channels(
        &mut self,
        abort_flag: Arc<AtomicBool>,
        msg_channel: Sender<ReactionMessage>,
    );

    fn search(&mut self, board: Board, depth: u8) -> Option<(Move, Eval)>;
}
