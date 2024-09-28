pub mod eval;
mod piece_square_table;
use eval::{Eval, EVAL_SCORES};
use piece_square_table::read_p_sq_table;

use crate::game::{
    bit_manipulation::{
        constants::{A_FILE, NOT_A_FILE, NOT_H_FILE},
        iter_set_bits,
    },
    board::{
        bit_board::BitBoard,
        move_gen::{
            attacks::direction_mask::{ALIGN_MASKS, CONNECTION_MASK},
            attacks_rook,
        },
    },
    Color, Piece, PieceType, Square,
};

use super::Board;

/// Evaluates the board state and returns a score in the perspective of the side to move.
pub fn evaluate_board(board: &Board) -> Eval {
    let mut ally_score = 0;
    let mut enemy_score = 0;

    let ally = board.side_to_move();
    let enemy = ally.opposite();

    let material_info_ally = MaterialInfo::from_board(board, ally);
    let material_info_enemy = MaterialInfo::from_board(board, enemy);

    ally_score += material_info_ally.total_material;
    enemy_score += material_info_enemy.total_material;

    // score += evaluate_pieces(board);

    //bishop pair bonus
    {
        if material_info_ally.num_pieces[PieceType::Bishop as usize] == 2 {
            let mut positions = board.get_piece_positions(Piece::new(PieceType::Bishop, ally));
            let first = positions.next();
            let second = positions.next();
            if first
                .zip(second)
                .is_some_and(|(a, b)| a.color() != b.color())
            {
                ally_score += EVAL_SCORES.bishop_pair;
            }
        }
        if material_info_enemy.num_pieces[PieceType::Bishop as usize] == 2 {
            let mut positions = board.get_piece_positions(Piece::new(PieceType::Bishop, enemy));
            let first = positions.next();
            let second = positions.next();
            if first
                .zip(second)
                .is_some_and(|(a, b)| a.color() != b.color())
            {
                enemy_score += EVAL_SCORES.bishop_pair;
            }
        }
    }

    //king pawn shield
    {
        let king_sq = board.get_king_pos(ally);
        let pawns = board.get_bb_pieces()[ally][PieceType::Pawn];
        let shield = PAWN_SHIELDS[ally][king_sq];
        ally_score += (pawns & shield).count_ones() as Eval * EVAL_SCORES.king_pawn_shield;
        let king_sq = board.get_king_pos(enemy);
        let pawns = board.get_bb_pieces()[enemy][PieceType::Pawn];
        let shield = PAWN_SHIELDS[enemy][king_sq];
        enemy_score += (pawns & shield).count_ones() as Eval * EVAL_SCORES.king_pawn_shield;
    }

    //king not castled penalty
    {
        if board.cur_state().castling_rights.has_any(ally) {
            ally_score += EVAL_SCORES.king_not_castled;
        }
        if board.cur_state().castling_rights.has_any(enemy) {
            enemy_score += EVAL_SCORES.king_not_castled;
        }
    }

    //double pawn / isolated pawn
    {
        let ally_pawns = *board.get_bb_pieces()[ally][PieceType::Pawn];
        let enemy_pawns = *board.get_bb_pieces()[enemy][PieceType::Pawn];
        for i in 0..8 {
            //double pawn
            let mask = A_FILE << i;
            ally_score +=
                ((ally_pawns & mask).count_ones().max(1) - 1) as Eval * EVAL_SCORES.doubled_pawn;
            enemy_score +=
                ((enemy_pawns & mask).count_ones().max(1) - 1) as Eval * EVAL_SCORES.doubled_pawn;

            //isolated pawn
            let mask_left = A_FILE << (i - 1).max(0);
            let mask_right = A_FILE << (i + 1).min(7);

            if (ally_pawns & mask) != 0 {
                let pawns_left = (ally_pawns & mask_left).count_ones();
                let pawns_right = (ally_pawns & mask_right).count_ones();
                if pawns_left == 0 && pawns_right == 0 {
                    ally_score += EVAL_SCORES.isolated_pawn;
                }
            }
            if (enemy_pawns & mask) != 0 {
                let pawns_left = (enemy_pawns & mask_left).count_ones();
                let pawns_right = (enemy_pawns & mask_right).count_ones();
                if pawns_left == 0 && pawns_right == 0 {
                    enemy_score += EVAL_SCORES.isolated_pawn;
                }
            }
        }
    }

    //rook file open / closed / semi open
    //rook on 7th
    {
        let ally_rooks = board.get_bb_pieces()[ally][PieceType::Rook];
        for sq in ally_rooks.get_occupied() {
            let mask = A_FILE << sq.file();
            let pawns = *board.get_bb_pieces()[ally][PieceType::Pawn];
            let pawns_enemy = *board.get_bb_pieces()[enemy][PieceType::Pawn];
            let pawns_on_file = (pawns | pawns_enemy) & mask;
            if pawns_on_file.count_ones() == 0 {
                ally_score += EVAL_SCORES.rook_open_file;
            } else if (mask & pawns) == 0 {
                ally_score += EVAL_SCORES.rook_semi_open_file;
            } else {
                ally_score += EVAL_SCORES.rook_on_closed;
            }

            let seventh_rank = match ally {
                Color::White => 6,
                Color::Black => 1,
            };

            if sq.rank() == seventh_rank {
                ally_score += EVAL_SCORES.rook_on_seventh;
            }

            if attacks_rook(sq, *ally_rooks, 0) != 0 {
                ally_score += EVAL_SCORES.rook_connected;
            }
        }

        let enemy_rooks = board.get_bb_pieces()[enemy][PieceType::Rook];
        for sq in enemy_rooks.get_occupied() {
            let mask = A_FILE << sq.file();
            let pawns = *board.get_bb_pieces()[enemy][PieceType::Pawn];
            let pawns_enemy = *board.get_bb_pieces()[ally][PieceType::Pawn];
            let pawns_on_file = (pawns | pawns_enemy) & mask;
            if pawns_on_file.count_ones() == 0 {
                enemy_score += EVAL_SCORES.rook_open_file;
            } else if (mask & pawns) == 0 {
                enemy_score += EVAL_SCORES.rook_semi_open_file;
            } else {
                enemy_score += EVAL_SCORES.rook_on_closed;
            }

            let seventh_rank = match enemy {
                Color::White => 6,
                Color::Black => 1,
            };

            if sq.rank() == seventh_rank {
                enemy_score += EVAL_SCORES.rook_on_seventh;
            }

            if attacks_rook(sq, *enemy_rooks, 0) != 0 {
                enemy_score += EVAL_SCORES.rook_connected;
            }
        }
    }

    let score = ally_score - enemy_score;
    score * ally.perspective() as Eval
}

///  Rook + Bishop + Knight + Queen:
const END_GAME_MATERIAL_START: Eval = 2050;

const PAWN_SHIELDS: [[u64; 64]; 2] = init_pawn_shields();

const fn init_pawn_shields() -> [[u64; 64]; 2] {
    let mut shields = [[0; 64]; 2];
    let mut i = 0;
    while i < 64 {
        let cur = 1 << i;
        let mut shield = 0;
        shield |= (cur << 7) & NOT_H_FILE;
        shield |= cur << 8;
        shield |= (cur << 9) & NOT_A_FILE;

        shields[0][i] = shield;
        let mut shield = 0;
        shield |= (cur >> 7) & NOT_A_FILE;
        shield |= cur >> 8;
        shield |= (cur >> 9) & NOT_H_FILE;

        shields[1][i] = shield;

        i += 1;
    }
    shields
}

fn calc_endgame_factor(sum: i32) -> f32 {
    1_f32 - 1_f32.min(sum as f32 / END_GAME_MATERIAL_START as f32)
}

struct MaterialInfo {
    num_pieces: [u8; 6],
    material_score: [Eval; 6],
    total_material: Eval,
}

impl MaterialInfo {
    fn from_board(board: &Board, color: Color) -> MaterialInfo {
        let mut num_pieces = [0; 6];
        let mut material_score = [0; 6];
        let mut total_material = 0;

        for piece in PieceType::iter() {
            let bb = board.get_bb_pieces()[color][piece];
            num_pieces[piece as usize] = bb.count_ones() as u8;
            let material = EVAL_SCORES.piece_values[piece] * num_pieces[piece as usize] as Eval;
            material_score[piece as usize] = material;
            total_material += material;
        }

        MaterialInfo {
            num_pieces,
            material_score,
            total_material,
        }
    }
}

#[inline(always)]
fn evaluate_pieces(board: &Board) -> Eval {
    let mut score = 0;

    let mut material_without_pawn = (0, 0);
    Square::iter_ah_18().for_each(|sq| {
        let piece = board.get_sq_piece(sq);
        if let Some(piece) = piece {
            let material = match piece.color() {
                Color::White => &mut material_without_pawn.0,
                Color::Black => &mut material_without_pawn.1,
            };

            *material += match piece.ptype() {
                PieceType::Knight => piece_square_table::KNIGHT_MG,
                PieceType::Bishop => piece_square_table::BISHOP_MG,
                PieceType::Rook => piece_square_table::ROOK_MG,
                PieceType::Queen => piece_square_table::QUEEN_MG,
                _ => 0,
            }
        }
    });

    let endgame_factor = (
        calc_endgame_factor(material_without_pawn.0),
        calc_endgame_factor(material_without_pawn.1),
    );
    let mid_game_factor = (1_f32 - endgame_factor.0, 1_f32 - endgame_factor.1);

    let bb = board.get_bb_pieces();
    for piece_type in PieceType::iter() {
        let white_piece_board = bb[Color::White][piece_type];
        let black_piece_board = bb[Color::Black][piece_type];

        let piece = piece_type.as_colored_piece(Color::White);
        iter_set_bits(*white_piece_board).for_each(|sq| {
            let w = read_p_sq_table(piece, sq);
            score += w.weight(mid_game_factor.0, endgame_factor.0);
        });
        let piece = piece_type.as_colored_piece(Color::Black);
        iter_set_bits(*black_piece_board).for_each(|sq| {
            let w = read_p_sq_table(piece, sq);
            score -= w.weight(mid_game_factor.0, endgame_factor.0);
        });
    }
    score
}
