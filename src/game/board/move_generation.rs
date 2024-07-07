use crate::{
    attack_pattern::{
        bishop_attacks_anti,
        bishop_attacks_main,
        rook_attacks_horizontal,
        rook_attacks_vertical,
        ATTACK_PATTERN_KING,
        ATTACK_PATTERN_KNIGHT,
        ATTACK_PATTERN_PAWN,
        MOVE_PATTERN_PAWN,
    },
    game::{ iter_set_bits, Color, Move, PieceVariation, Square },
};

use super::Board;

impl Board {
    /// Generates all the pseudo legal moves for a given square index.
    /// Pseudo legal moves do not consider moves that get your king into check.
    /// All moves are normale moves. After taking the move, it must be transformed into a different move
    /// type if it is a promotion, en passant or castling move.
    pub fn get_pseudo_legal_moves(&self, square: u8) -> Option<Vec<Move>> {
        let piece = self.get_piece(square)?;
        let sq = Square::from(square);
        let mut moves = Vec::new();
        let possible_moves = match piece.0 {
            PieceVariation::PAWN => {
                let mut possible_moves = MOVE_PATTERN_PAWN[piece.1][square as usize];

                possible_moves ^= possible_moves & self.get_all_pieces_bb();

                // check for 2 square moves
                if square / 8 == 1 && possible_moves != 0 && piece.1 == Color::WHITE {
                    let moves = MOVE_PATTERN_PAWN[piece.1][(square + 8) as usize];
                    possible_moves |= moves ^ (moves & self.get_all_pieces_bb());
                }
                if square / 8 == 6 && possible_moves != 0 && piece.1 == Color::BLACK {
                    let moves = MOVE_PATTERN_PAWN[piece.1][(square - 8) as usize];
                    possible_moves |= moves ^ (moves & self.get_all_pieces_bb());
                }

                let mut attack_moves = ATTACK_PATTERN_PAWN[piece.1][square as usize];
                attack_moves ^= attack_moves & self.get_color_pieces_bb(&piece.1);
                if Square::valid(self.en_passant) {
                    attack_moves &=
                        self.get_color_pieces_bb(&piece.1.opposite()) |
                        Square::to_board_bit(self.en_passant);
                } else {
                    attack_moves &= self.get_color_pieces_bb(&piece.1.opposite());
                }
                possible_moves |= attack_moves;

                possible_moves
            }
            PieceVariation::KNIGHT => {
                let mut possible_moves = ATTACK_PATTERN_KNIGHT[square as usize];

                possible_moves ^= possible_moves & self.get_color_pieces_bb(&piece.1);

                possible_moves
            }
            PieceVariation::ROOK => {
                let mut possible_moves = 0;

                let enemy = self.get_color_pieces_bb(&piece.1.opposite());
                let ally = self.get_color_pieces_bb(&piece.1);

                possible_moves |= rook_attacks_vertical(enemy, ally, sq);
                possible_moves |= rook_attacks_horizontal(enemy, ally, sq);
                possible_moves
            }
            PieceVariation::BISHOP => {
                let mut possible_moves = 0;

                let enemy = self.get_color_pieces_bb(&piece.1.opposite());
                let ally = self.get_color_pieces_bb(&piece.1);

                possible_moves |= bishop_attacks_main(enemy, ally, sq);
                possible_moves |= bishop_attacks_anti(enemy, ally, sq);
                possible_moves
            }
            PieceVariation::QUEEN => {
                let mut possible_moves = 0;

                let enemy = self.get_color_pieces_bb(&piece.1.opposite());
                let ally = self.get_color_pieces_bb(&piece.1);

                possible_moves |= rook_attacks_vertical(enemy, ally, sq);
                possible_moves |= rook_attacks_horizontal(enemy, ally, sq);
                possible_moves |= bishop_attacks_main(enemy, ally, sq);
                possible_moves |= bishop_attacks_anti(enemy, ally, sq);
                possible_moves
            }
            PieceVariation::KING => {
                let mut possible_moves = ATTACK_PATTERN_KING[square as usize];

                possible_moves ^= possible_moves & self.get_color_pieces_bb(&piece.1);

                // Castling
                let (king_side, queen_side) = match piece.1 {
                    Color::WHITE => self.white_castle,
                    Color::BLACK => self.black_castle,
                };
                if
                    king_side &&
                    self.get_piece(Square::F1.into()).is_none() &&
                    self.get_piece(Square::G1.into()).is_none()
                {
                    possible_moves |= Square::to_board_bit(Square::G1.into());
                }
                if
                    queen_side &&
                    self.get_piece(Square::D1.into()).is_none() &&
                    self.get_piece(Square::C1.into()).is_none() &&
                    self.get_piece(Square::B1.into()).is_none()
                {
                    possible_moves |= Square::to_board_bit(Square::B1.into());
                }

                possible_moves
            }
        };

        moves.extend(
            iter_set_bits(possible_moves).map(|dest| { Move::normal(square, dest, piece, None) })
        );

        return Some(moves);
    }
}
