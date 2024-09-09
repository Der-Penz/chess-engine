use std::collections::HashSet;

use crate::{bot::INFINITY_DEPTH, game::Board};

use super::searcher::MAX_QS_DEPTH;

const MAX_REPETITION_DEPTH: u8 = max_repetition_depth();
const fn max_repetition_depth() -> u8 {
    INFINITY_DEPTH + MAX_QS_DEPTH
}

/// #### RepetitionHistory
/// Basic structure to keep track of the board hashes during the search.  
/// This is used to detect repetitions.  
/// For simplicity, the repetition history will raise a Draw if the same position is repeated twice.
pub struct RepetitionHistory {
    b_history: HashSet<u64>,
    searched_positions: [u64; MAX_REPETITION_DEPTH as usize],
    starting_index: [i8; MAX_REPETITION_DEPTH as usize],
    pointer: usize,
}

impl RepetitionHistory {
    pub fn new() -> RepetitionHistory {
        RepetitionHistory {
            b_history: HashSet::new(),
            searched_positions: [0; MAX_REPETITION_DEPTH as usize],
            starting_index: [-1; MAX_REPETITION_DEPTH as usize],
            pointer: 0,
        }
    }

    /// Initialize the repetition history with the current board state.
    pub fn init(&mut self, board: &Board) {
        self.b_history.clear();
        self.b_history.extend(board.repetition_history());
        self.pointer = 0;
    }

    /// Check if the given hash was played before.
    pub fn two_fold_repetition(&mut self, zobrist: u64) -> bool {
        if self.pointer == 0 || self.starting_index[self.pointer - 1] == -1 {
            self.b_history.contains(&zobrist)
                || self.searched_positions[..self.pointer].contains(&zobrist)
        } else {
            self.searched_positions[self.starting_index[self.pointer - 1] as usize..self.pointer]
                .contains(&zobrist)
        }
    }

    /// Push the current board hash to the repetition history.
    pub fn push_hash(&mut self, zobrist: u64, capture_or_pawn_move: bool) {
        self.searched_positions[self.pointer] = zobrist;
        if capture_or_pawn_move {
            self.starting_index[self.pointer] = self.pointer as i8;
        } else {
            if self.pointer == 0 {
                self.starting_index[self.pointer] = -1;
            } else {
                self.starting_index[self.pointer] = self.starting_index[self.pointer - 1];
            }
        }
        self.pointer += 1;
    }

    /// Pop the last board hash from the repetition history.
    pub fn pop_hash(&mut self) {
        self.pointer -= 1;
    }
}
