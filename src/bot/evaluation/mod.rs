pub mod eval;
mod piece_square_table;
use eval::Eval;
use piece_square_table::read_p_sq_table;

use crate::game::{bit_manipulation::iter_set_bits, Color, PieceType, Square};

use super::Board;

/// Evaluates the board state and returns a score in the perspective of the side to move.
pub fn evaluate_board(board: &Board) -> Eval {
    let mut score = 0;

    let color = board.side_to_move();

    score += evaluate_pieces(board);

    score * color.perspective() as Eval
}

///  Rook + Bishop + Knight + Queen:
const END_GAME_MATERIAL_START: Eval = 2050;

fn calc_endgame_factor(sum: i32) -> f32 {
    1_f32 - 1_f32.min(sum as f32 / END_GAME_MATERIAL_START as f32)
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
