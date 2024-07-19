use std::fmt::{Debug, Display};

use thiserror::Error;

use crate::game::{bb_to_string, CastleType, Color, Square};

use super::{BitBoardOperation, Board};

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{:?} FEN: {}\n",
            bb_to_string(|sq| { self.get_field_piece(sq.into()).map(|p| p.to_string()) }),
            self.color_to_move,
            self.to_fen()
        )
    }
}

#[derive(Debug, Error)]
pub enum FENError {
    #[error("Error Parsing FEN String")]
    ParsingError,
    #[error("Missing Group in FEN String")]
    MissingGroup,
}

impl Board {
    pub fn from_fen(fen_string: &str) -> Result<Board, FENError> {
        let mut board = Board::empty();

        let mut splits = fen_string.split_whitespace();

        let fen_group = splits.next().ok_or(FENError::MissingGroup)?;

        let mut row: i8 = 7;
        let mut col: i8 = 0;
        for char in fen_group.chars() {
            if char != '/' && (row < 0 || col > 7) {
                Err(FENError::ParsingError)?;
            }
            match char {
                '/' => {
                    row -= 1;
                    col = 0;
                }
                number if number.is_digit(10) => {
                    col += number.to_digit(10).ok_or(FENError::ParsingError)? as i8;
                }
                'R' | 'N' | 'B' | 'Q' | 'K' | 'P' | 'r' | 'n' | 'b' | 'q' | 'k' | 'p' => {
                    board.update_bb(&char.into(), (row * 8 + col) as u8, BitBoardOperation::SET);
                    col += 1;
                }

                _ => Err(FENError::ParsingError)?,
            }
        }

        let fen_group = splits.next().ok_or(FENError::MissingGroup)?;

        match fen_group {
            "w" => {
                board.color_to_move = Color::WHITE;
            }
            "b" => {
                board.color_to_move = Color::BLACK;
            }
            _ => Err(FENError::ParsingError)?,
        }

        let fen_group = splits.next().ok_or(FENError::MissingGroup)?;

        for char in fen_group.chars() {
            match char {
                'K' => {
                    board.castle_rights[Color::WHITE][CastleType::KingSide] = true;
                }
                'Q' => {
                    board.castle_rights[Color::WHITE][CastleType::QueenSide] = true;
                }
                'k' => {
                    board.castle_rights[Color::BLACK][CastleType::KingSide] = true;
                }
                'q' => {
                    board.castle_rights[Color::BLACK][CastleType::QueenSide] = true;
                }
                '-' => (),
                _ => Err(FENError::ParsingError)?,
            }
        }

        let fen_group = splits.next().ok_or(FENError::MissingGroup)?;

        match fen_group {
            "-" => {
                board.en_passant = None;
            }
            _ => {
                let mut chars = fen_group.chars();
                let en_passant = (chars.next().ok_or(FENError::ParsingError)? as u8) - ('a' as u8);
                let en_passant = ((chars
                    .next()
                    .ok_or(FENError::ParsingError)?
                    .to_digit(10)
                    .ok_or(FENError::ParsingError)? as u8)
                    - 1)
                    * 8
                    + en_passant;
                Square::valid(en_passant)
                    .then_some(0)
                    .ok_or(FENError::ParsingError)?;

                board.en_passant = Some(Square::from(en_passant));
            }
        }

        let fen_group = splits.next().ok_or(FENError::MissingGroup)?;
        let moves: usize = fen_group.parse().ok().ok_or(FENError::ParsingError)?;
        board.half_move_clock = moves;

        let fen_group = splits.next().ok_or(FENError::MissingGroup)?;
        let moves: usize = fen_group.parse().ok().ok_or(FENError::ParsingError)?;
        board.move_number = moves;

        Ok(board)
    }

    pub fn to_fen(&self) -> String {
        let mut s = String::new();

        let mut empty = 0;
        Square::iter_ah_81().for_each(|square| {
            let piece = self.get_field_piece(square.into());
            match piece {
                Some(piece) => {
                    if empty > 0 {
                        s.push_str(&format!("{}", empty));
                        empty = 0;
                    }
                    s.push_str(&format!("{}", piece.1.transform_char(&piece.0.as_char())));
                }
                None => {
                    empty += 1;
                }
            }
            if square.file() == 7 {
                if empty > 0 {
                    s.push_str(&format!("{}", empty));
                }
                empty = 0;
                if (square as u8) != 7 {
                    s.push('/');
                }
            }
        });

        s.push_str(&format!(" {} ", self.color_to_move));

        let mut castle_rights = String::new();
        if self.castle_rights[Color::WHITE][CastleType::KingSide] {
            castle_rights.push('K');
        }
        if self.castle_rights[Color::WHITE][CastleType::QueenSide] {
            castle_rights.push('Q');
        }
        if self.castle_rights[Color::BLACK][CastleType::KingSide] {
            castle_rights.push('k');
        }
        if self.castle_rights[Color::BLACK][CastleType::QueenSide] {
            castle_rights.push('q');
        }

        if castle_rights.is_empty() {
            s.push_str("-");
        } else {
            s.push_str(&castle_rights);
        }

        match self.en_passant {
            Some(sq) => s.push_str(&format!(" {} ", sq.to_string())),
            None => s.push_str(" - "),
        }

        s.push_str(&format!("{} {}", self.half_move_clock, self.move_number));

        s
    }
}
