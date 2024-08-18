mod piece_square_table;
use crate::game::{bit_manipulation::iter_set_bits, castle_rights::CastleType, Color, PieceType};

use super::Board;

const SCORE_CASTLE_RIGHT: i32 = 40;

const SCORE_PAWN: i32 = 10;
const SCORE_KNIGHT: i32 = 30;
const SCORE_BISHOP: i32 = 30;
const SCORE_ROOK: i32 = 50;
const SCORE_QUEEN: i32 = 90;

const SCORE_PIECES: [i32; 6] = [
    SCORE_PAWN,
    SCORE_KNIGHT,
    SCORE_BISHOP,
    SCORE_ROOK,
    SCORE_QUEEN,
    1, //multiplier for king (1 king * 1 = 1) Score won't be effected by this
];

pub fn evaluate_board(board: &Board) -> i32 {
    let mut score = 0;

    score += evaluate_pieces(board);
    score += evaluate_castle(board);

    score / 10
}

fn evaluate_castle(board: &Board) -> i32 {
    let mut score = 0;
    let castle_rights = &board.cur_state().castling_rights;
    castle_rights
        .has(Color::White, CastleType::KingSide)
        .then(|| score += SCORE_CASTLE_RIGHT);
    castle_rights
        .has(Color::White, CastleType::QueenSide)
        .then(|| score += SCORE_CASTLE_RIGHT);

    castle_rights
        .has(Color::Black, CastleType::KingSide)
        .then(|| score -= SCORE_CASTLE_RIGHT);
    castle_rights
        .has(Color::Black, CastleType::QueenSide)
        .then(|| score -= SCORE_CASTLE_RIGHT);

    score
}

fn evaluate_pieces(board: &Board) -> i32 {
    let mut score = 0;

    let bb = board.get_bb_pieces();
    for piece_type in PieceType::iter() {
        let piece_score = SCORE_PIECES[piece_type];
        let white_piece_board = bb[Color::White][piece_type];
        let black_piece_board = bb[Color::Black][piece_type];
        score += piece_score
            * (white_piece_board.count_ones() as i32 - black_piece_board.count_ones() as i32);

        iter_set_bits(*white_piece_board).for_each(|sq| {
            let index = Color::White.relative_sq(sq);
            score += piece_square_table::PIECE_SQUARE_TABLES[piece_type][index as usize] as i32;
        });
        iter_set_bits(*black_piece_board).for_each(|sq| {
            let index = Color::Black.relative_sq(sq);
            score -= piece_square_table::PIECE_SQUARE_TABLES[piece_type][index as usize] as i32;
        });
    }
    score
}
