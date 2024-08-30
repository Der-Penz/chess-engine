mod diagnostics;
use std::sync::{atomic::AtomicBool, mpsc::Sender, Arc};

use crate::game::{Board, Move};

use super::ReactionMessage;

pub mod min_max;

pub type AbortFlag = Arc<AtomicBool>;

pub trait Search {
    fn search(
        &mut self,
        board: Board,
        depth: u8,
        flag: &AbortFlag,
        msg_channel: &mut Sender<ReactionMessage>,
    ) -> Option<(Move, i64)>;
}
