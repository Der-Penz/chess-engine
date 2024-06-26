use crate::game::{next_set_bit, Color, PieceVariation};

use super::Board;


impl Board {
    pub fn generate_pseudo_legal_moves(& self, color: Color) -> u64 {
        let boards = self.get_boards(color);
        let mut moves: u64 = 0;

        for (piece, _i) in PieceVariation::iter() {
            let mut board = self.white_boards[piece];
            while let Some(index) = next_set_bit(board) {
                moves |= piece.attack_pattern(color)[index];
                board ^= 1 << index;
            }
        }

        return moves;
    }
}
