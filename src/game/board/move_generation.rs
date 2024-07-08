use crate::{ attack_pattern, game::{ iter_set_bits, Color, Move, PieceVariation, Square } };

use super::Board;

impl Board {
    /// Generates all the pseudo legal moves for a given square index.
    /// Pseudo legal moves do not consider moves that get your king into check.
    /// All moves are normale moves. After taking the move, it must be transformed into a different move
    /// type if it is a promotion, en passant or castling move.
    pub fn get_pseudo_legal_moves(&self, square: u8) -> Option<Vec<Move>> {
        let piece = self.get_piece(square)?;
        let sq = Square::from(square);

        let ally = self.get_bb_color_occupied(&piece.1);
        let enemy = self.get_bb_color_occupied(&piece.1.opposite());

        let mut moves = Vec::new();
        let possible_moves = match piece.0 {
            PieceVariation::PAWN =>
                Board::attacks_pawn(square, enemy, ally, &piece.1, self.en_passant),
            PieceVariation::KNIGHT => Board::attacks_knight(square, ally),
            PieceVariation::ROOK => Board::attacks_rook(&sq, enemy, ally),
            PieceVariation::BISHOP => Board::attacks_bishop(&sq, enemy, ally),
            PieceVariation::QUEEN => Board::attacks_queen(&sq, enemy, ally),
            PieceVariation::KING => Board::attacks_king(square, ally, &piece.1, self),
        };

        moves.extend(
            iter_set_bits(possible_moves).map(|dest| { Move::normal(square, dest, piece, None) })
        );

        return Some(moves);
    }

    fn attacks_rook(sq: &Square, enemy: u64, ally: u64) -> u64 {
        let mut attacks = 0;

        attacks |= attack_pattern::rook_attacks_vertical(enemy, ally, *sq);
        attacks |= attack_pattern::rook_attacks_horizontal(enemy, ally, *sq);
        attacks
    }

    fn attacks_bishop(sq: &Square, enemy: u64, ally: u64) -> u64 {
        let mut attacks = 0;

        attacks |= attack_pattern::bishop_attacks_main(enemy, ally, *sq);
        attacks |= attack_pattern::bishop_attacks_anti(enemy, ally, *sq);
        attacks
    }

    fn attacks_queen(sq: &Square, enemy: u64, ally: u64) -> u64 {
        let mut attacks = 0;

        attacks |= Board::attacks_rook(sq, enemy, ally);
        attacks |= Board::attacks_bishop(sq, enemy, ally);
        attacks
    }

    fn attacks_king(sq: u8, ally: u64, color: &Color, board: &Board) -> u64 {
        let mut attacks = attack_pattern::ATTACK_PATTERN_KING[sq as usize];

        attacks ^= attacks & ally;

        // Castling
        let (king_side, queen_side) = match color {
            Color::WHITE => board.white_castle,
            Color::BLACK => board.black_castle,
        };
        if
            king_side &&
            board.get_piece(Square::F1.into()).is_none() &&
            board.get_piece(Square::G1.into()).is_none()
        {
            attacks |= Square::to_board_bit(Square::G1.into());
        }
        if
            queen_side &&
            board.get_piece(Square::D1.into()).is_none() &&
            board.get_piece(Square::C1.into()).is_none() &&
            board.get_piece(Square::B1.into()).is_none()
        {
            attacks |= Square::to_board_bit(Square::B1.into());
        }
        attacks
    }

    fn attacks_knight(sq: u8, ally: u64) -> u64 {
        let mut attacks = attack_pattern::ATTACK_PATTERN_KNIGHT[sq as usize];

        attacks ^= attacks & ally;
        attacks
    }

    fn attacks_pawn(sq: u8, enemy: u64, ally: u64, color: &Color, en_passant: u8) -> u64 {
        let mut attacks = attack_pattern::MOVE_PATTERN_PAWN[*color][sq as usize];

        let all = enemy | ally;
        attacks ^= attacks & all;

        // check for 2 square moves
        if sq / 8 == 1 && attacks != 0 && *color == Color::WHITE {
            let moves = attack_pattern::MOVE_PATTERN_PAWN[*color][(sq + 8) as usize];
            attacks |= moves ^ (moves & all);
        }
        if sq / 8 == 6 && attacks != 0 && *color == Color::BLACK {
            let moves = attack_pattern::MOVE_PATTERN_PAWN[*color][(sq - 8) as usize];
            attacks |= moves ^ (moves & all);
        }

        let mut attack_moves = attack_pattern::ATTACK_PATTERN_PAWN[*color][sq as usize];
        attack_moves ^= attack_moves & ally;
        if Square::valid(en_passant) {
            attack_moves &= enemy | Square::to_board_bit(en_passant);
        } else {
            attack_moves &= enemy;
        }
        attacks |= attack_moves;

        attacks
    }
}
