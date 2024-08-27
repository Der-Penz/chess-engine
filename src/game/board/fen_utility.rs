use itertools::Itertools;

use crate::game::{castle_rights::CastleRights, color::Color, piece::Piece, square::Square};

use super::{board_error::FENError, zobrist::ZOBRIST, Board};

// FEN struct to handle conversion between FEN strings and boards
pub struct FENUtility();

impl FENUtility {
    pub const START_FEN: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    pub fn from_fen(fen_string: &str) -> Result<Board, FENError> {
        let mut board = Board::empty();

        let mut splits = fen_string.split_whitespace();

        let fen_group = splits
            .next()
            .ok_or(FENError::MissingGroup("Figure position"))?;

        let mut row: i8 = 7;
        let mut col: i8 = 0;
        for char in fen_group.chars() {
            if char != '/' && (row < 0 || col > 7) {
                return Err(FENError::ParsingError(format!(
                    "{} is no valid character",
                    char
                )));
            }
            match char {
                '/' => {
                    row -= 1;
                    col = 0;
                }
                number if number.is_digit(10) => {
                    col += number
                        .to_digit(10)
                        .ok_or(FENError::ParsingError(format!("number is not base 10")))?
                        as i8;
                }
                piece_char @ ('R' | 'N' | 'B' | 'Q' | 'K' | 'P' | 'r' | 'n' | 'b' | 'q' | 'k'
                | 'p') => {
                    let square = Square::try_from((row * 8 + col) as u8).map_err(|_| {
                        FENError::ParsingError(format!("Invalid square {}", row * 8 + col))
                    })?;

                    board.update_bb(piece_char.into(), square, true);
                    col += 1;
                }
                _ => {
                    return Err(FENError::ParsingError(format!(
                        "{} is no valid character",
                        char
                    )))
                }
            }
        }

        board
            .get_piece_positions(Piece::WHITE_KING)
            .exactly_one()
            .map_err(|_| FENError::MissingKing(Color::White))?;
        board
            .get_piece_positions(Piece::BLACK_KING)
            .exactly_one()
            .map_err(|_| FENError::MissingKing(Color::Black))?;

        let fen_group = splits
            .next()
            .ok_or(FENError::MissingGroup("Side to move"))?;

        match fen_group {
            color @ ("w" | "b") => {
                board.side_to_move = Color::from_char(color.chars().next().unwrap())
            }
            _ => {
                return Err(FENError::ParsingError(format!(
                    "Color group must either be w or b not {}",
                    fen_group
                )))
            }
        }

        let fen_group = splits.next().ok_or(FENError::MissingGroup("Castling"))?;

        board.current_state.castling_rights = CastleRights::try_from(fen_group)
            .map_err(|err| FENError::ParsingError(err.to_string()))?;

        let fen_group = splits
            .next()
            .ok_or(FENError::MissingGroup("en passant square"))?;

        match fen_group {
            "-" => {
                board.current_state.en_passant = None;
            }
            square @ _ => {
                let square = Square::try_from(square).map_err(|_| {
                    FENError::ParsingError(format!("{} not a valid square", square))
                })?;

                if square.rank() != 2 && square.rank() != 5 {
                    return Err(FENError::ParsingError(format!(
                        "{} not a valid square",
                        square
                    )));
                }

                board.current_state.en_passant = Some(square);
            }
        }

        let fen_group = splits
            .next()
            .ok_or(FENError::MissingGroup("Half move clock"))?;
        let ply_clock: u8 = fen_group.parse().map_err(|_| {
            FENError::ParsingError(format!("{} not a valid positive number", fen_group))
        })?;
        board.current_state.ply_clock = ply_clock;

        let fen_group = splits.next().ok_or(FENError::MissingGroup("Move number"))?;
        let moves: usize = fen_group.parse().map_err(|_| {
            FENError::ParsingError(format!("{} not a valid positive number", fen_group))
        })?;
        board.ply_count = match board.side_to_move {
            Color::White => moves * 2 - 1,
            Color::Black => moves * 2,
        };

        let zobrist = ZOBRIST.calculate_zobrist_key(&board);
        board.current_state.zobrist = zobrist;

        Ok(board)
    }

    // Convert the board to a FEN string
    pub fn to_fen(board: &Board) -> String {
        let mut fen_string = String::with_capacity(64); // try to avoid reallocations
        let mut empty_squares = 0;
        Square::iter_ah_81().for_each(|square| {
            let piece = board.get_sq_piece(square);
            match piece {
                Some(piece) => {
                    if empty_squares > 0 {
                        fen_string.push_str(&format!("{empty_squares}"));
                        empty_squares = 0;
                    }
                    fen_string.push_str(&format!("{piece}"));
                }
                None => {
                    empty_squares += 1;
                }
            }
            if square.file() == 7 {
                if empty_squares > 0 {
                    fen_string.push_str(&format!("{empty_squares}"));
                }
                empty_squares = 0;
                if square != Square::H1 {
                    fen_string.push('/');
                }
            }
        });

        fen_string.push_str(&format!(" {} ", board.side_to_move));
        fen_string.push_str(&board.current_state.castling_rights.to_string());

        match board.current_state.en_passant {
            Some(square) => fen_string.push_str(&format!(" {square} ")),
            None => fen_string.push_str(" - "),
        }

        fen_string.push_str(&format!("{}", board.current_state.ply_clock));
        fen_string.push_str(&format!(" {}", board.ply_count() / 2));

        fen_string
    }
}
