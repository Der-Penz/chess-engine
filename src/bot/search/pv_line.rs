use crate::{bot::INFINITY_DEPTH, game::Move};

/// A principal variation line.
/// Handles the best line of moves found by the search algorithm.
pub struct PVLine {
    moves: [Move; INFINITY_DEPTH as usize],
    count_move: usize,
}

impl std::default::Default for PVLine {
    fn default() -> Self {
        Self {
            moves: [Move::default(); INFINITY_DEPTH as usize],
            count_move: 0,
        }
    }
}

impl PVLine {
    pub fn add(&mut self, mv: Move) {
        self.moves[self.count_move] = mv;
        self.count_move += 1;
    }

    pub fn reset(&mut self) {
        self.count_move = 0;
    }

    pub fn moves(&self) -> &[Move] {
        &self.moves[..self.count_move]
    }

    pub fn len(&self) -> usize {
        self.count_move
    }

    pub fn get_move(&self, index: usize) -> Option<&Move> {
        self.moves[..self.count_move].get(index)
    }
}

impl std::fmt::Display for PVLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.count_move {
            write!(f, "{} ", self.moves[i])?;
        }
        Ok(())
    }
}
