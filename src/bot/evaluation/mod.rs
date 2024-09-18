pub mod eval;
mod piece_square_table;
use eval::Eval;
use piece_square_table::read_p_sq_table;

use crate::game::{
    bit_manipulation::iter_set_bits,
    board::move_gen::{LegalMoveList, MoveGeneration},
    castle_rights::CastleRights,
    Color, PieceType, Square,
};

use super::Board;

const SCORE_CASTLE_RIGHT: Eval = 10;
const CASTLE_SCORE_BY_INDEX: [Eval; 16] = [
    0,                      //0000
    SCORE_CASTLE_RIGHT,     //0001
    SCORE_CASTLE_RIGHT,     //0010
    SCORE_CASTLE_RIGHT * 2, //0011
    -SCORE_CASTLE_RIGHT,    //0100
    0,                      //0101
    0,                      //0110
    SCORE_CASTLE_RIGHT,     //0111
    -SCORE_CASTLE_RIGHT,    //1000
    0,                      //1001
    0,                      //1010
    SCORE_CASTLE_RIGHT,     //1011
    SCORE_CASTLE_RIGHT * 2, //1100
    -SCORE_CASTLE_RIGHT,    //1101
    -SCORE_CASTLE_RIGHT,    //1110
    0,                      //1111
];

const MOBILITY_SCORE: Eval = 1;

/// Evaluates the board state and returns a score in the perspective of the side to move.
pub fn evaluate_board(board: &Board, precomputed_moves: Option<&LegalMoveList>) -> Eval {
    let mut score = 0;

    let board_state = board.cur_state();
    let color = board.side_to_move();

    score += evaluate_pieces(board);
    // score += evaluate_castle(&board_state.castling_rights);

    //mobility score
    // let move_count = match precomputed_moves {
    //     Some(moves) => moves.len() as Eval,
    //     None => MoveGeneration::generate_legal_moves(board).len() as Eval,
    // };
    // score += (move_count >> MOBILITY_SCORE) * color.perspective() as Eval;

    //score is from the perspective of the white player so we need to negate if it is black to movee
    (score / 100) * color.perspective() as Eval
}

#[inline(always)]
fn evaluate_castle(castle_rights: &CastleRights) -> Eval {
    CASTLE_SCORE_BY_INDEX[castle_rights.as_u8() as usize]
}

///  Rook + Bishop + Knight + Queen :
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
