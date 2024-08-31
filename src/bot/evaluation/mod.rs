pub mod eval;
mod piece_square_table;
use eval::Eval;
use piece_square_table::read_p_sq_table;

use crate::game::{
    bit_manipulation::iter_set_bits,
    board::move_gen::{LegalMoveList, MoveGeneration},
    castle_rights::CastleRights,
    Color, PieceType,
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

const SCORE_PAWN: Eval = 10;
const SCORE_KNIGHT: Eval = 30;
const SCORE_BISHOP: Eval = 30;
const SCORE_ROOK: Eval = 50;
const SCORE_QUEEN: Eval = 90;

const SCORE_PIECES: [Eval; 6] = [
    SCORE_PAWN,
    SCORE_KNIGHT,
    SCORE_BISHOP,
    SCORE_ROOK,
    SCORE_QUEEN,
    1, //multiplier for king (1 king * 1 = 1) Score won't be effected by this
];

const MOBILITY_SCORE: Eval = 1;

/// Evaluates the board state and returns a score in the perspective of the side to move.
pub fn evaluate_board(board: &Board, precomputed_moves: Option<&LegalMoveList>) -> Eval {
    let mut score = 0;

    let board_state = board.cur_state();
    let color = board.side_to_move();

    score += evaluate_pieces(board);
    score += evaluate_castle(&board_state.castling_rights);

    //mobility score
    let move_count = match precomputed_moves {
        Some(moves) => moves.len() as Eval,
        None => MoveGeneration::generate_legal_moves(board).len() as Eval,
    };
    score += (move_count >> MOBILITY_SCORE) * color.perspective() as Eval;

    (score / 10) * color.perspective() as Eval
}

#[inline(always)]
fn evaluate_castle(castle_rights: &CastleRights) -> Eval {
    CASTLE_SCORE_BY_INDEX[castle_rights.as_u8() as usize]
}

#[inline(always)]
fn evaluate_pieces(board: &Board) -> Eval {
    let mut score = 0;

    let bb = board.get_bb_pieces();
    for piece_type in PieceType::iter() {
        let piece_score = SCORE_PIECES[piece_type];
        let white_piece_board = bb[Color::White][piece_type];
        let black_piece_board = bb[Color::Black][piece_type];
        score += piece_score
            * (white_piece_board.count_ones() as Eval - black_piece_board.count_ones() as Eval);

        let piece = piece_type.as_colored_piece(Color::White);
        iter_set_bits(*white_piece_board).for_each(|sq| {
            score += read_p_sq_table(piece, sq);
        });
        let piece = piece_type.as_colored_piece(Color::Black);
        iter_set_bits(*black_piece_board).for_each(|sq| {
            score -= read_p_sq_table(piece, sq);
        });
    }
    score
}
