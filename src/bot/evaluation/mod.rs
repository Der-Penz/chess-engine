mod piece_square_table;
use crate::game::{
    bit_manipulation::iter_set_bits,
    board::move_gen::{LegalMoveList, MoveGeneration},
    castle_rights::CastleRights,
    Color, PieceType,
};

use super::Board;

const SCORE_CASTLE_RIGHT: i64 = 40;
const CASTLE_SCORE_BY_INDEX: [i64; 16] = [
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

const SCORE_PAWN: i64 = 10;
const SCORE_KNIGHT: i64 = 30;
const SCORE_BISHOP: i64 = 30;
const SCORE_ROOK: i64 = 50;
const SCORE_QUEEN: i64 = 90;

const SCORE_PIECES: [i64; 6] = [
    SCORE_PAWN,
    SCORE_KNIGHT,
    SCORE_BISHOP,
    SCORE_ROOK,
    SCORE_QUEEN,
    1, //multiplier for king (1 king * 1 = 1) Score won't be effected by this
];

const MOBILITY_SCORE: i64 = 2;

/// Evaluates the board state and returns a score. Positive score indicates white is winning, negative score indicates black is winning.
pub fn evaluate_board(board: &Board, precomputed_moves: Option<&LegalMoveList>) -> i64 {
    let mut score = 0;

    let board_state = board.cur_state();
    let color = board.side_to_move();

    score += evaluate_pieces(board);
    score += evaluate_castle(&board_state.castling_rights);

    //mobility score
    let move_count = match precomputed_moves {
        Some(moves) => moves.len() as i64,
        None => MoveGeneration::generate_legal_moves(board).len() as i64,
    };
    score += move_count * MOBILITY_SCORE * color.perspective() as i64;

    score / 10
}

#[inline(always)]
fn evaluate_castle(castle_rights: &CastleRights) -> i64 {
    CASTLE_SCORE_BY_INDEX[castle_rights.as_u8() as usize]
}

#[inline(always)]
fn evaluate_pieces(board: &Board) -> i64 {
    let mut score = 0;

    let bb = board.get_bb_pieces();
    for piece_type in PieceType::iter() {
        let piece_score = SCORE_PIECES[piece_type];
        let white_piece_board = bb[Color::White][piece_type];
        let black_piece_board = bb[Color::Black][piece_type];
        score += piece_score
            * (white_piece_board.count_ones() as i64 - black_piece_board.count_ones() as i64);

        iter_set_bits(*white_piece_board).for_each(|sq| {
            let index = Color::White.relative_sq(sq);
            score += piece_square_table::PIECE_SQUARE_TABLES[piece_type][index as usize] as i64;
        });
        iter_set_bits(*black_piece_board).for_each(|sq| {
            let index = Color::Black.relative_sq(sq);
            score -= piece_square_table::PIECE_SQUARE_TABLES[piece_type][index as usize] as i64;
        });
    }
    score
}
