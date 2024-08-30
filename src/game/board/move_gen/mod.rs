mod helper;
mod legal_move_list;
pub mod magic;
pub mod test;
use attacks::direction_mask::{get_direction_mask, Direction, ALIGN_MASKS};
use helper::{MoveGenerationData, MoveGenerationMasks};
pub use legal_move_list::LegalMoveList;
use magic::get_rook_moves;

use crate::game::{
    castle_rights::{CastleRights, CastleType},
    color::Color,
    move_notation::MoveFlag,
    piece_type::PieceType,
    square::Square,
};

use super::{bit_board::BitBoard, Board};

pub mod attacks;

pub struct MoveGeneration();

impl MoveGeneration {
    pub fn generate_legal_moves(board: &Board) -> LegalMoveList {
        Self::generate_moves(board, false)
    }

    pub fn generate_legal_moves_captures(board: &Board) -> LegalMoveList {
        Self::generate_moves(board, true)
    }

    fn generate_moves(board: &Board, captures_only: bool) -> LegalMoveList {
        let mut legal_moves = LegalMoveList::default();

        let data = MoveGenerationData::from_board(&board);
        let mut masks = MoveGenerationMasks::default();
        masks.calculate_king_danger(&data, &board);

        let move_type_mask = if captures_only { data.enemy } else { 0u64 };
        //only king moves are allowed if in multi check (no other moves are allowed or castling)
        let king_moves = attacks_king(data.king_sq, data.ally) & !masks.king_danger;

        legal_moves.create_and_add_moves(
            data.king_sq,
            king_moves,
            MoveFlag::Normal,
            move_type_mask,
        );
        if masks.multi_check {
            legal_moves.set_masks(masks);
            return legal_moves;
        }

        masks.calculate_pins(&data, board);
        masks.calculate_push_and_capture(&data, board);

        //calculate moves for pinned pieces
        if !masks.in_check {
            for sq in BitBoard::from(masks.orthogonal_pinned).get_occupied() {
                let piece = board.get_sq_piece(sq).expect("Pinned piece must exist");
                let pin_move_mask = ALIGN_MASKS[sq][data.king_sq];

                match piece.ptype() {
                    PieceType::Pawn => {
                        let double_push = moves_pawn_double_push(sq, data.occupied, data.color);
                        legal_moves.create_and_add_moves(
                            sq,
                            double_push & pin_move_mask,
                            MoveFlag::DoublePawnPush,
                            move_type_mask,
                        );
                        let pawn_move = moves_pawn(sq, data.enemy, data.ally, data.color);
                        legal_moves.create_and_add_pawn_moves(
                            sq,
                            pawn_move & pin_move_mask,
                            data.color,
                            move_type_mask,
                        );
                    }
                    //if queen is straight pinned, it can only move straight
                    PieceType::Rook | PieceType::Queen => {
                        let moves = attacks_rook(sq, data.enemy, data.ally);
                        legal_moves.create_and_add_moves(
                            sq,
                            moves & pin_move_mask,
                            MoveFlag::Normal,
                            move_type_mask,
                        );
                    }
                    PieceType::Knight | PieceType::Bishop => (), //can't move if pinned by a rook like piece
                    PieceType::King => unreachable!("King can't be pinned"),
                };
            }
            for sq in BitBoard::from(masks.diagonal_pinned).get_occupied() {
                let piece = board.get_sq_piece(sq).expect("Pinned piece must exist");
                let pin_move_mask = ALIGN_MASKS[sq][data.king_sq];

                match piece.ptype() {
                    PieceType::Pawn => {
                        let en_passant = attacks_pawn_en_passant(
                            sq,
                            data.color,
                            board.cur_state().en_passant.as_ref(),
                            data.king_sq,
                            data.enemy,
                            data.ally,
                            board,
                        );
                        legal_moves.create_and_add_moves(
                            sq,
                            en_passant & pin_move_mask,
                            MoveFlag::EnPassant,
                            move_type_mask,
                        );
                        let attacks = attacks_pawn(sq, data.enemy, data.ally, data.color);
                        legal_moves.create_and_add_pawn_moves(
                            sq,
                            attacks & pin_move_mask,
                            data.color,
                            move_type_mask,
                        );
                    }
                    //if queen is diagonal pinned, it can only move diagonally
                    PieceType::Bishop | PieceType::Queen => {
                        let moves = attacks_bishop(sq, data.enemy, data.ally);
                        legal_moves.create_and_add_moves(
                            sq,
                            moves & pin_move_mask,
                            MoveFlag::Normal,
                            move_type_mask,
                        );
                    }
                    PieceType::Knight | PieceType::Rook => (), //can't move if pinned by a bishop like piece
                    PieceType::King => unreachable!("King can't be pinned"),
                };
            }
        }

        //calculate moves for non-pinned pieces
        let non_pinned = data.ally & !(masks.diagonal_pinned | masks.orthogonal_pinned);
        for sq in BitBoard::from(non_pinned).get_occupied() {
            let piece = board
                .get_sq_piece(sq)
                .expect(format!("Piece at {} must exist", sq).as_str());

            match piece.ptype() {
                PieceType::Pawn => {
                    let pawn_move = moves_pawn(sq, data.enemy, data.ally, data.color);
                    legal_moves.create_and_add_pawn_moves(
                        sq,
                        pawn_move & masks.push_capture_mask,
                        data.color,
                        move_type_mask,
                    );

                    let attacks = attacks_pawn(sq, data.enemy, data.ally, data.color);
                    legal_moves.create_and_add_pawn_moves(
                        sq,
                        attacks & masks.push_capture_mask,
                        data.color,
                        move_type_mask,
                    );

                    let double_push = moves_pawn_double_push(sq, data.occupied, data.color);
                    legal_moves.create_and_add_moves(
                        sq,
                        double_push & masks.push_capture_mask,
                        MoveFlag::DoublePawnPush,
                        move_type_mask,
                    );
                    let en_passant = attacks_pawn_en_passant(
                        sq,
                        data.color,
                        board.cur_state().en_passant.as_ref(),
                        data.king_sq,
                        data.enemy,
                        data.ally,
                        board,
                    );
                    legal_moves.create_and_add_moves(
                        sq,
                        en_passant & masks.push_mask,
                        MoveFlag::EnPassant,
                        move_type_mask,
                    );
                    if en_passant != 0 {
                        let en_passant_pawn = board.cur_state().en_passant.unwrap().square_value()
                            as i8
                            - data.color.perspective() * 8;

                        if masks.checkers & (1u64 << en_passant_pawn) != 0 {
                            legal_moves.create_and_add_moves(
                                sq,
                                en_passant,
                                MoveFlag::EnPassant,
                                move_type_mask,
                            );
                        }
                    }
                }
                PieceType::Knight => {
                    let moves = attacks_knight(sq, data.ally);
                    legal_moves.create_and_add_moves(
                        sq,
                        moves & masks.push_capture_mask,
                        MoveFlag::Normal,
                        move_type_mask,
                    );
                }
                PieceType::Bishop => {
                    let moves = attacks_bishop(sq, data.enemy, data.ally);
                    legal_moves.create_and_add_moves(
                        sq,
                        moves & masks.push_capture_mask,
                        MoveFlag::Normal,
                        move_type_mask,
                    );
                }
                PieceType::Rook => {
                    let moves = attacks_rook(sq, data.enemy, data.ally);
                    legal_moves.create_and_add_moves(
                        sq,
                        moves & masks.push_capture_mask,
                        MoveFlag::Normal,
                        move_type_mask,
                    );
                }
                PieceType::Queen => {
                    let moves = attacks_queen(sq, data.enemy, data.ally);
                    legal_moves.create_and_add_moves(
                        sq,
                        moves & masks.push_capture_mask,
                        MoveFlag::Normal,
                        move_type_mask,
                    );
                }
                PieceType::King => {
                    if !masks.in_check {
                        if board
                            .cur_state()
                            .castling_rights
                            .has(data.color, CastleType::KingSide)
                        {
                            legal_moves.create_and_add_moves(
                                sq,
                                moves_king_castle_king_side(
                                    sq,
                                    data.occupied,
                                    masks.king_danger,
                                    data.color,
                                ),
                                MoveFlag::KingSideCastle,
                                move_type_mask,
                            );
                        }
                        if board
                            .cur_state()
                            .castling_rights
                            .has(data.color, CastleType::QueenSide)
                        {
                            legal_moves.create_and_add_moves(
                                sq,
                                moves_king_castle_queen_side(
                                    sq,
                                    data.occupied,
                                    masks.king_danger,
                                    data.color,
                                ),
                                MoveFlag::QueenSideCastle,
                                move_type_mask,
                            );
                        }
                    }
                }
            }
        }

        legal_moves.set_masks(masks);
        legal_moves
    }
}

#[inline(always)]
pub fn attacks_rook(sq: Square, enemy: u64, ally: u64) -> u64 {
    (*magic::get_rook_moves(sq, enemy | ally)) & !ally
}

#[inline(always)]
pub fn attacks_bishop(sq: Square, enemy: u64, ally: u64) -> u64 {
    (*magic::get_bishop_moves(sq, enemy | ally)) & !ally
}

#[inline(always)]
fn attacks_queen(sq: Square, enemy: u64, ally: u64) -> u64 {
    ((*magic::get_bishop_moves(sq, enemy | ally)) & !ally)
        | ((*magic::get_rook_moves(sq, enemy | ally)) & !ally)
}

#[inline(always)]
pub fn attacks_knight(sq: Square, ally: u64) -> u64 {
    attacks::ATTACK_PATTERN_KNIGHT[sq] & !ally
}

#[inline(always)]
pub fn attacks_king(sq: Square, ally: u64) -> u64 {
    attacks::ATTACK_PATTERN_KING[sq] & !ally
}

fn moves_king_castle_queen_side(sq: Square, occupied: u64, attacked: u64, color: Color) -> u64 {
    if sq != CastleType::KING_SOURCE[color] {
        return 0;
    }

    let queen_side_free = attacks::CASTLE_FREE_SQUARES[color][CastleType::QueenSide];
    let queen_side_attack_free = attacks::CASTLE_ATTACK_FREE_SQUARES[color][CastleType::QueenSide];

    let queen_side_possible =
        (queen_side_free & occupied) == 0 && (queen_side_attack_free & attacked) == 0;

    if queen_side_possible {
        CastleType::KING_DEST[CastleRights::to_index(color, CastleType::QueenSide) as usize]
            .to_mask()
    } else {
        0
    }
}

fn moves_king_castle_king_side(sq: Square, occupied: u64, attacked: u64, color: Color) -> u64 {
    if sq != CastleType::KING_SOURCE[color] {
        return 0;
    }

    let king_side_free = attacks::CASTLE_FREE_SQUARES[color][CastleType::KingSide];
    let king_side_attack_free = attacks::CASTLE_ATTACK_FREE_SQUARES[color][CastleType::KingSide];

    let king_side_possible =
        (king_side_free & occupied) == 0 && (king_side_attack_free & attacked) == 0;

    if king_side_possible {
        CastleType::KING_DEST[CastleRights::to_index(color, CastleType::KingSide) as usize]
            .to_mask()
    } else {
        0
    }
}

#[inline(always)]
pub fn attacks_pawn(sq: Square, enemy: u64, ally: u64, color: Color) -> u64 {
    (attacks::ATTACK_PATTERN_PAWN[color][sq] & !ally) & enemy
}

#[inline(always)]
fn moves_pawn_double_push(sq: Square, occupied: u64, color: Color) -> u64 {
    if sq.rank() != color.pawn_rank() {
        return 0;
    }
    let index = (sq.square_value() as i8 + (color.perspective() * 8)) as u8;
    if occupied & (1 << index) != 0 {
        return 0;
    }

    attacks::MOVE_PATTERN_PAWN[color][index as usize] & !occupied
}

#[inline(always)]
fn attacks_pawn_en_passant(
    sq: Square,
    color: Color,
    en_passant: Option<&Square>,
    king_sq: Square,
    enemy: u64,
    ally: u64,
    board: &Board,
) -> u64 {
    if en_passant.is_none() {
        return 0;
    }
    let en_passant_square = en_passant.unwrap();

    let enemy_pawn_mask =
        1u64 << (en_passant_square.square_value() as i8 - (color.perspective() * 8));
    let sq_mask = sq.to_mask();

    //check for if the en passant capture would expose a discovered attack on the king
    //by removing the pawn and the en passant pawn from the board, we can check if there
    //is a rook or queen attacking the king by using the rook horizontal attack pattern from the king square
    //and check if this ray would attack a enemy rook or queen
    let enemy_without_pawn = enemy & !(enemy_pawn_mask);
    let ally_without_pawn = ally & !(sq_mask);

    let mut rank_attack_ray =
        *get_rook_moves(king_sq, enemy_without_pawn | ally_without_pawn) & !ally_without_pawn;
    rank_attack_ray &= get_direction_mask(king_sq, Direction::WestToEast);

    if king_sq.rank() == sq.rank() && rank_attack_ray & enemy_without_pawn != 0 {
        let horizontal_rook_attack = BitBoard::from(rank_attack_ray & enemy_without_pawn)
            .get_occupied()
            .any(|square| {
                let piece = board.get_sq_piece(square).map(|piece| piece.ptype());

                piece == Some(PieceType::Rook) || piece == Some(PieceType::Queen)
            });

        //if there is a rook or queen attacking the king, the en passant is invalid
        if horizontal_rook_attack {
            return 0;
        }
    }

    attacks::ATTACK_PATTERN_PAWN[color][sq] & en_passant_square.to_mask()
}

#[inline(always)]
pub fn moves_pawn(sq: Square, enemy: u64, ally: u64, color: Color) -> u64 {
    attacks::MOVE_PATTERN_PAWN[color][sq] & !(ally | enemy)
}
