use crate::game::{
    castle_rights::{CastleRights, CastleType},
    color::Color,
    move_notation::{Move, MoveFlag},
    piece::Piece,
    piece_type::PieceType,
    square::Square,
};

use super::{bit_board::BitBoard, board_state::BoardState, Board};

pub mod attack_pattern;

pub struct MoveGeneration();

impl MoveGeneration {
    pub fn attacks_rook(sq: &Square, enemy: u64, ally: u64) -> u64 {
        let mut attacks = 0;
        attacks |= attack_pattern::rook_attacks_vertical(enemy, ally, *sq);
        attacks |= attack_pattern::rook_attacks_horizontal(enemy, ally, *sq);
        attacks
    }

    pub fn attacks_bishop(sq: &Square, enemy: u64, ally: u64) -> u64 {
        let mut attacks = 0;
        attacks |= attack_pattern::bishop_attacks_main(enemy, ally, *sq);
        attacks |= attack_pattern::bishop_attacks_anti(enemy, ally, *sq);
        attacks
    }

    pub fn attacks_queen(sq: &Square, enemy: u64, ally: u64) -> u64 {
        let mut attacks = 0;
        attacks |= Self::attacks_rook(sq, enemy, ally);
        attacks |= Self::attacks_bishop(sq, enemy, ally);
        attacks
    }

    pub fn attacks_knight(sq: &Square, ally: u64) -> u64 {
        attack_pattern::ATTACK_PATTERN_KNIGHT[*sq] & !ally
    }

    pub fn attacks_king(sq: &Square, ally: u64) -> u64 {
        attack_pattern::ATTACK_PATTERN_KING[*sq] & !ally
    }

    /// only validates free squares for castling not if castling is possible or castling through check
    pub fn moves_king_castle(sq: &Square, enemy: u64, ally: u64, color: &Color) -> u64 {
        if sq != &CastleType::KING_SOURCE[*color] {
            return 0;
        }

        let all = ally | enemy;
        let king_side_free = attack_pattern::CASTLE_FREE_SQUARES[*color][CastleType::KingSide];
        let queen_side_free = attack_pattern::CASTLE_FREE_SQUARES[*color][CastleType::QueenSide];

        let king_side_possible = king_side_free & all == 0;
        let queen_side_possible = queen_side_free & all == 0;

        match (king_side_possible, queen_side_possible) {
            (true, true) => {
                CastleType::KING_DEST[CastleRights::to_index(color, &CastleType::KingSide) as usize]
                    .to_mask()
                    | CastleType::KING_DEST
                        [CastleRights::to_index(color, &CastleType::QueenSide) as usize]
                        .to_mask()
            }
            (true, false) => CastleType::KING_DEST
                [CastleRights::to_index(color, &CastleType::KingSide) as usize]
                .to_mask(),
            (false, true) => CastleType::KING_DEST
                [CastleRights::to_index(color, &CastleType::QueenSide) as usize]
                .to_mask(),
            (false, false) => 0,
        }
    }

    pub fn attacks_pawn(sq: &Square, enemy: u64, ally: u64, color: &Color) -> u64 {
        (attack_pattern::ATTACK_PATTERN_PAWN[*color][*sq] & !ally) & enemy
    }

    pub fn moves_pawn_double_push(sq: &Square, enemy: u64, ally: u64, color: &Color) -> u64 {
        if sq.rank() != color.pawn_rank() {
            return 0;
        }
        let index = (sq.square_value() as i8 + (color.perspective() * 8)) as u8;
        let all = BitBoard::new(ally | enemy);
        if all.is_occupied(&Square::new(index)) {
            return 0;
        }

        attack_pattern::MOVE_PATTERN_PAWN[*color][index as usize] & !*all
    }

    pub fn attacks_pawn_en_passant(sq: &Square, color: &Color, en_passant: Option<&Square>) -> u64 {
        if en_passant.is_none() {
            return 0;
        } else {
            let en_passant_square = en_passant.unwrap();
            attack_pattern::ATTACK_PATTERN_PAWN[*color][*sq] & en_passant_square.to_mask()
        }
    }

    pub fn moves_pawn(sq: &Square, enemy: u64, ally: u64, color: &Color) -> u64 {
        attack_pattern::MOVE_PATTERN_PAWN[*color][*sq] & !(ally | enemy)
    }

    /// Generates pseudo-legal moves for a given piece on a given square
    pub fn generate_pseudo_legal_moves(
        square: &Square,
        piece: &Piece,
        enemy: &BitBoard,
        ally: &BitBoard,
        board_state: &BoardState,
    ) -> Option<Vec<Move>> {
        let mut moves: Vec<Move> = Vec::new();

        let enemy = *enemy;
        let ally = *ally;

        match piece.ptype() {
            PieceType::Pawn => {
                let attacks = Self::attacks_pawn(square, *enemy, *ally, &piece.color());
                BitBoard::from(attacks).get_occupied().for_each(|sq| {
                    if sq.rank() == piece.color().promotion_rank() {
                        MoveFlag::PAWN_PROMOTION_FLAGS.into_iter().for_each(|flag| {
                            moves.push(Move::new(*square, sq, flag));
                        });
                    } else {
                        moves.push(Move::new(*square, sq, MoveFlag::Normal));
                    }
                });

                if square.rank() == piece.color().pawn_rank() {
                    let double_push =
                        Self::moves_pawn_double_push(square, *enemy, *ally, &piece.color());
                    BitBoard::from(double_push).get_occupied().for_each(|sq| {
                        moves.push(Move::new(*square, sq, MoveFlag::DoublePawnPush));
                    });
                }

                let en_passant_attack = Self::attacks_pawn_en_passant(
                    square,
                    &piece.color(),
                    board_state.en_passant.as_ref(),
                );
                BitBoard::from(en_passant_attack)
                    .get_occupied()
                    .for_each(|sq| {
                        moves.push(Move::new(*square, sq, MoveFlag::EnPassant));
                    });

                let pawn_moves = Self::moves_pawn(square, *enemy, *ally, &piece.color());
                BitBoard::from(pawn_moves).get_occupied().for_each(|sq| {
                    if sq.rank() == piece.color().promotion_rank() {
                        MoveFlag::PAWN_PROMOTION_FLAGS.into_iter().for_each(|flag| {
                            moves.push(Move::new(*square, sq, flag));
                        });
                    } else {
                        moves.push(Move::new(*square, sq, MoveFlag::Normal));
                    }
                });
            }
            PieceType::Knight => {
                let attacks = Self::attacks_knight(square, *ally);
                BitBoard::from(attacks).get_occupied().for_each(|sq| {
                    moves.push(Move::new(*square, sq, MoveFlag::Normal));
                });
            }
            PieceType::Bishop => {
                let attacks = Self::attacks_bishop(square, *enemy, *ally);
                BitBoard::from(attacks).get_occupied().for_each(|sq| {
                    moves.push(Move::new(*square, sq, MoveFlag::Normal));
                });
            }
            PieceType::Rook => {
                let attacks = Self::attacks_rook(square, *enemy, *ally);
                BitBoard::from(attacks).get_occupied().for_each(|sq| {
                    moves.push(Move::new(*square, sq, MoveFlag::Normal));
                });
            }
            PieceType::Queen => {
                let mut attacks = Self::attacks_rook(square, *enemy, *ally);
                attacks |= Self::attacks_bishop(square, *enemy, *ally);
                BitBoard::from(attacks).get_occupied().for_each(|sq| {
                    moves.push(Move::new(*square, sq, MoveFlag::Normal));
                });
            }
            PieceType::King => {
                let attacks = Self::attacks_king(square, *ally);
                BitBoard::from(attacks).get_occupied().for_each(|sq| {
                    moves.push(Move::new(*square, sq, MoveFlag::Normal));
                });

                let castle_moves = Self::moves_king_castle(square, *enemy, *ally, &piece.color());
                BitBoard::from(castle_moves).get_occupied().for_each(|sq| {
                    match sq {
                        s if s == CastleType::KING_DEST[0] || s == CastleType::KING_DEST[2] => {
                            moves.push(Move::new(*square, sq, MoveFlag::KingSideCastle));
                        }
                        s if s == CastleType::KING_DEST[1] || s == CastleType::KING_DEST[3] => {
                            moves.push(Move::new(*square, sq, MoveFlag::QueenSideCastle));
                        }
                        _ => (),
                    };
                });
            }
        };

        Some(moves)
    }

    /// Generates all legal moves for the current board state
    pub fn generate_all_moves(board: &Board) -> Vec<Move> {
        let bb = board.get_bb_occupied(&board.side_to_move);
        let mut moves = Vec::with_capacity(218); // 218 is the maximum number of moves possible in a position

        bb.get_occupied().for_each(|sq| {
            let piece = board
                .get_sq_piece(&sq)
                .expect("There has to be a piece on this board or the bb are out of sync");
            let enemy = board.get_bb_occupied(&board.side_to_move.opposite());
            let ally = board.get_bb_occupied(&board.side_to_move);

            let sq_moves =
                Self::generate_pseudo_legal_moves(&sq, &piece, &enemy, &ally, &board.current_state);

            if let Some(sq_moves) = sq_moves {
                moves.extend(sq_moves);
            }
        });

        //TODO filter out illegal moves
        Self::filter_legal_moves(moves, board)
    }

    pub fn filter_legal_moves(moves: Vec<Move>, board: &Board) -> Vec<Move> {
        moves
            .into_iter()
            .filter(|m| {
                let mut board = board.clone();
                let correct = board.make_move(m, false, false);

                if correct.is_err() {
                    return false;
                }
                let in_check = board.in_check(&board.side_to_move.opposite());

                !in_check
            })
            .collect()
    }
}
