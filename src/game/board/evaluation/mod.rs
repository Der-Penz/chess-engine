mod piece_square_table;
use super::Board;
use crate::game::{iter_set_bits, CastleType, Color, PieceVariation};

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

    board
        .castle_rights
        .has(&Color::WHITE, &CastleType::KingSide)
        .then(|| score += SCORE_CASTLE_RIGHT);
    board
        .castle_rights
        .has(&Color::WHITE, &CastleType::QueenSide)
        .then(|| score += SCORE_CASTLE_RIGHT);

    board
        .castle_rights
        .has(&Color::BLACK, &CastleType::KingSide)
        .then(|| score -= SCORE_CASTLE_RIGHT);
    board
        .castle_rights
        .has(&Color::BLACK, &CastleType::QueenSide)
        .then(|| score -= SCORE_CASTLE_RIGHT);

    score
}

fn evaluate_pieces(board: &Board) -> i32 {
    let mut score = 0;

    for (_, i) in PieceVariation::iter() {
        let piece_score = SCORE_PIECES[i];
        let white_piece_board = board.white_boards[i];
        let black_piece_board = board.black_boards[i];
        score += piece_score
            * (white_piece_board.count_ones() as i32 - black_piece_board.count_ones() as i32);

        iter_set_bits(white_piece_board).for_each(|sq| {
            let index = Color::WHITE.relative(sq);
            score += piece_square_table::PIECE_SQUARE_TABLES[i][index as usize] as i32;
        });
        iter_set_bits(black_piece_board).for_each(|sq| {
            let index = Color::BLACK.relative(sq);
            score -= piece_square_table::PIECE_SQUARE_TABLES[i][index as usize] as i32;
        });
    }
    score
}
