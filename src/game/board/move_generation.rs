use crate::game::{
    display_position, iter_set_bits, next_set_bit, to_board_bit, valid_position, Color, Move, PieceVariation
};

use super::Board;

impl Board {

    /**
        Generates all the pseudo legal moves for a given board index.
        Pseudo legal moves do not consider moves that get your king into check
     */
    pub fn get_pseudo_legal_moves(&self, pos: u8) -> Option<Vec<Move>> {
        let piece = self.get_piece(pos)?;
        let mut moves = Vec::new();
        match piece.0 {
            PieceVariation::PAWN => {
                let attack_pattern = PieceVariation::PAWN.attack_pattern(piece.1);
                let mut possible_moves = attack_pattern[pos as usize];

                possible_moves ^= possible_moves & self.get_pieces_board(&piece.1);

                // check for 2 square moves
                if pos / 8 == 1 {
                    if self.get_field_color(pos + 8).is_some_and(|c| c == piece.1) {
                        possible_moves &= !to_board_bit(pos + 16);
                    }
                }

                moves.extend(iter_set_bits(possible_moves).map(|dest| {
                    Move::normal(pos, dest, piece)
                }));

                // if the position is valid it
                if valid_position(self.en_passant) {
                    //en passant can only happen on the 4th or 5th rank. This constrain should
                    //already be inferred while setting the square
                    assert!([4, 5].contains(&(self.en_passant / 8)));

                    let col = self.en_passant % 8;
                    if col > 0 && col - 1 == pos % 8 {
                        moves.push(Move::en_passant(pos, pos + 9, piece.1));
                    }
                    if col < 7 && col + 1 == pos % 8 {
                        moves.push(Move::en_passant(pos, pos + 7, piece.1));
                    }
                }

                // display_position(possible_moves, to_board_bit(pos));
                return Some(moves);
            }
            PieceVariation::KNIGHT => todo!("Knight moves not yet implemented"),
            PieceVariation::BISHOP => todo!("Bishop moves not yet implemented"),
            PieceVariation::ROOK => todo!("Rook moves not yet implemented"),
            PieceVariation::QUEEN => todo!("Queen moves not yet implemented"),
            PieceVariation::KING => todo!("King moves not yet implemented"),
        }

        None
    }
}
