use crate::{
    attack_pattern::{ ATTACK_PATTERN_PAWN, MOVE_PATTERN_PAWN },
    game::{ iter_set_bits, Color, Move, PieceVariation },
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
                let mut possible_moves = MOVE_PATTERN_PAWN[piece.1][pos as usize];

                possible_moves ^= possible_moves & self.get_pieces_bb(&piece.1);

                // check for 2 square moves
                if pos / 8 == 1 && possible_moves != 0 && piece.1 == Color::WHITE {
                    let moves = MOVE_PATTERN_PAWN[piece.1][(pos + 8) as usize];
                    possible_moves |= moves ^ (moves & self.get_all_pieces_bb());
                }
                if pos / 8 == 6 && possible_moves != 0 && piece.1 == Color::BLACK {
                    let moves = MOVE_PATTERN_PAWN[piece.1][(pos - 8) as usize];
                    possible_moves |= moves ^ (moves & self.get_all_pieces_bb());
                }

                let mut attack_moves = ATTACK_PATTERN_PAWN[piece.1][pos as usize];
                attack_moves ^= attack_moves & self.get_pieces_bb(&piece.1);
                attack_moves &= self.get_pieces_bb(&piece.1.opposite());
                possible_moves |= attack_moves;
                moves.extend(
                    iter_set_bits(possible_moves).map(|dest| { Move::normal(pos, dest, piece) })
                );

                //TODO: en passant
                //TODO: handle promotion moves (if it should return a single move or multiple moves or if it should be handled later on)
                return Some(moves);
            }
            PieceVariation::KNIGHT => todo!("Knight moves not yet implemented"),
            PieceVariation::BISHOP => todo!("Bishop moves not yet implemented"),
            PieceVariation::ROOK => todo!("Rook moves not yet implemented"),
            PieceVariation::QUEEN => todo!("Queen moves not yet implemented"),
            PieceVariation::KING => todo!("King moves not yet implemented"),
        }
    }
}
