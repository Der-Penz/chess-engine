use crate::{
    attack_pattern,
    game::{
        iter_set_bits, BaseMoveType, Color, Move, Piece, PieceVariation, PromotionPiece, Square,
    },
};

use super::Board;

impl Board {
    /// Returns all possible moves for the current player.
    pub fn get_all_possible_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let pieces = self.get_bb_color_occupied(&self.color_to_move);
        for square in iter_set_bits(pieces) {
            if let Some(m) = self.get_pseudo_legal_moves(square) {
                moves.extend(m);
            }
        }
        self.filter_legal_moves(moves)
    }

    /// Generates all the pseudo legal moves for a given square index.
    /// Pseudo legal moves do not consider moves that get your king into check.
    /// All moves are normale moves. After taking the move, it must be transformed into a different move
    /// type if it is a promotion, en passant or castling move.
    pub fn get_pseudo_legal_moves(&self, square: u8) -> Option<Vec<Move>> {
        let piece = self.get_field_piece(square)?;
        let sq = Square::from(square);

        let ally = self.get_bb_color_occupied(&piece.1);
        let enemy = self.get_bb_color_occupied(&piece.1.opposite());

        let mut moves = Vec::new();
        let possible_moves = match piece.0 {
            PieceVariation::PAWN => {
                Board::attacks_pawn(square, enemy, ally, &piece.1, self.en_passant)
            }
            PieceVariation::KNIGHT => Board::attacks_knight(square, ally),
            PieceVariation::ROOK => Board::attacks_rook(&sq, enemy, ally),
            PieceVariation::BISHOP => Board::attacks_bishop(&sq, enemy, ally),
            PieceVariation::QUEEN => Board::attacks_queen(&sq, enemy, ally),
            PieceVariation::KING => Board::attacks_king(square, ally, &piece.1, self, true),
        };

        //handle promotion
        if piece.0 == PieceVariation::PAWN {
            let promotion_rank = match piece.1 {
                Color::WHITE => 7,
                Color::BLACK => 0,
            };
            moves.extend(
                iter_set_bits(possible_moves)
                    .map(|dest| {
                        if Square::from(dest).rank() == promotion_rank {
                            return vec![
                                Move::new(
                                    square,
                                    dest,
                                    BaseMoveType::Promotion(PromotionPiece::Knight),
                                ),
                                Move::new(
                                    square,
                                    dest,
                                    BaseMoveType::Promotion(PromotionPiece::Bishop),
                                ),
                                Move::new(
                                    square,
                                    dest,
                                    BaseMoveType::Promotion(PromotionPiece::Rook),
                                ),
                                Move::new(
                                    square,
                                    dest,
                                    BaseMoveType::Promotion(PromotionPiece::Queen),
                                ),
                            ];
                        } else {
                            return vec![Move::new(square, dest, BaseMoveType::Normal)];
                        }
                    })
                    .flatten(),
            );
        } else {
            moves.extend(
                iter_set_bits(possible_moves)
                    .map(|dest| Move::new(square, dest, BaseMoveType::Normal)),
            );
        }

        return Some(moves);
    }

    pub fn filter_legal_moves(&self, moves: Vec<Move>) -> Vec<Move> {
        moves
            .into_iter()
            .filter(|m| {
                let mut board = self.clone();
                let played = board.play_move(m, false);
                if played.is_err() {
                    return false;
                }
                let (white_check, black_check) = board.in_check();

                if white_check.is_none() && black_check.is_none() {
                    return false;
                }

                match board.color_to_move {
                    Color::BLACK => !white_check.unwrap(),
                    Color::WHITE => !black_check.unwrap(),
                }
            })
            .collect()
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

    fn attacks_king(sq: u8, ally: u64, color: &Color, board: &Board, castle_check: bool) -> u64 {
        let mut attacks = attack_pattern::ATTACK_PATTERN_KING[sq as usize];

        attacks ^= attacks & ally;

        if !castle_check {
            return attacks;
        }

        let [king_side, queen_side] = board.castle_rights[*color];

        //checks if the path is free and not attacked
        fn castle_allowed(path: &Vec<Square>, board: &Board, color: &Color) -> bool {
            for path in path {
                if board.get_field_piece((*path).into()).is_some() {
                    return false;
                }
                if board.square_attacked(*path, color.opposite()) {
                    return false;
                }
            }
            true
        }

        if king_side {
            let path = match color {
                Color::WHITE => vec![Square::F1, Square::G1],
                Color::BLACK => vec![Square::F8, Square::G8],
            };
            if castle_allowed(&path, board, color) {
                attacks |= Square::to_board_bit(path[1].into());
            }
        }

        if queen_side {
            let path = match color {
                Color::WHITE => vec![Square::D1, Square::C1, Square::B1],
                Color::BLACK => vec![Square::D8, Square::C8, Square::B8],
            };
            if castle_allowed(&path, board, color) {
                attacks |= Square::to_board_bit(path[1].into());
            }
        }

        attacks
    }

    fn attacks_knight(sq: u8, ally: u64) -> u64 {
        let mut attacks = attack_pattern::ATTACK_PATTERN_KNIGHT[sq as usize];

        attacks ^= attacks & ally;
        attacks
    }

    fn attacks_pawn(
        sq: u8,
        enemy: u64,
        ally: u64,
        color: &Color,
        en_passant: Option<Square>,
    ) -> u64 {
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
        if en_passant.is_some() {
            attack_moves &= enemy | Square::to_board_bit(en_passant.unwrap().into());
        } else {
            attack_moves &= enemy;
        }
        attacks |= attack_moves;

        attacks
    }

    /// Whether a square is attacked by a color or not.
    pub fn square_attacked(&self, square: Square, attacked_by: Color) -> bool {
        let sq = square.into();
        let ally = self.get_bb_color_occupied(&attacked_by.opposite());
        let enemy = self.get_bb_color_occupied(&attacked_by);

        let pawns = self.get_bb_for(&Piece(PieceVariation::PAWN, attacked_by));
        if (Board::attacks_pawn(sq, enemy, ally, &attacked_by.opposite(), self.en_passant) & pawns)
            != 0
        {
            return true;
        }

        let knight = self.get_bb_for(&Piece(PieceVariation::KNIGHT, attacked_by));
        if (Board::attacks_knight(sq, ally) & knight) != 0 {
            return true;
        }

        let king = self.get_bb_for(&Piece(PieceVariation::KING, attacked_by));
        if (Board::attacks_king(sq, ally, &attacked_by.opposite(), &self, false) & king) != 0 {
            return true;
        }

        let square = Square::from(sq);
        let bishop_queen = self.get_bb_for(&Piece(PieceVariation::BISHOP, attacked_by))
            | self.get_bb_for(&Piece(PieceVariation::QUEEN, attacked_by));
        if (Board::attacks_bishop(&square, enemy, ally) & bishop_queen) != 0 {
            return true;
        }

        let rook_queen = self.get_bb_for(&Piece(PieceVariation::ROOK, attacked_by))
            | self.get_bb_for(&Piece(PieceVariation::QUEEN, attacked_by));
        if (Board::attacks_rook(&square, enemy, ally) & rook_queen) != 0 {
            return true;
        }

        false
    }

    /// Whether a player is in check or not. (white_check, black_check).
    /// If no king is found, the Option will be None.
    pub fn in_check(&self) -> (Option<bool>, Option<bool>) {
        let mut white_check = None;
        let mut black_check = None;

        let king: u64 = self.white_boards[PieceVariation::KING];
        let king_sq = iter_set_bits(king).next();
        if let Some(king_sq) = king_sq {
            white_check = self.square_attacked(king_sq.into(), Color::BLACK).into();
        }

        let king: u64 = self.white_boards[PieceVariation::KING];
        let king_sq = iter_set_bits(king).next();
        if let Some(king_sq) = king_sq {
            white_check = self.square_attacked(king_sq.into(), Color::BLACK).into();
        }

        let king: u64 = self.black_boards[PieceVariation::KING];
        let king_sq = iter_set_bits(king).next();
        if let Some(king_sq) = king_sq {
            black_check = self.square_attacked(king_sq.into(), Color::WHITE).into();
        }

        (white_check, black_check)
    }

    pub fn in_check_color(&self, color: Color) -> Option<bool> {
        let (white, black) = self.in_check();

        match color {
            Color::WHITE => white,
            Color::BLACK => black,
        }
    }
}
