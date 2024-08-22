use std::sync::mpsc::Sender;

use crate::game::{Board, Move};

use super::{AbortFlag, Message};

pub mod min_max;

pub trait Search {
    fn search(&mut self, board: Board, depth: u8, flag: &mut AbortFlag) -> Option<(Move, i64)>;
}
