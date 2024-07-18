use std::fmt::{ Debug, Display };

use thiserror::Error;

use crate::game::{ bb_to_string, Color, Square };

use super::{ BitBoardOperation, Board };

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
                    board.update_bb(
                        &char.into(),
                        (row * 8 + col) as u8,
                        BitBoardOperation::SET
                    );
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

        board.black_castle = (false, false);
        board.white_castle = (false, false);
        for char in fen_group.chars() {
            match char {
                'K' => {
                    board.white_castle.0 = true;
                }
                'Q' => {
                    board.white_castle.1 = true;
                }
                'k' => {
                    board.black_castle.0 = true;
                }
                'q' => {
                    board.black_castle.1 = true;
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
                let en_passant =
                    ((
                        chars
                            .next()
                            .ok_or(FENError::ParsingError)?
                            .to_digit(10)
                            .ok_or(FENError::ParsingError)? as u8
                    ) -
                        1) *
                        8 +
                    en_passant;
                Square::valid(en_passant).then_some(0).ok_or(FENError::ParsingError)?;

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

        if self.white_castle.0 {
            s.push('K');
        }
        if self.white_castle.1 {
            s.push('Q');
        }
        if self.black_castle.0 {
            s.push('k');
        }
        if self.black_castle.1 {
            s.push('q');
        }

        if
            !self.black_castle.0 ||
            !self.black_castle.1 ||
            !self.white_castle.0 ||
            !self.white_castle.1
        {
            s.push_str("-");
        }

        match self.en_passant {
            Some(sq) => s.push_str(&format!(" {} ", sq.to_string())),
            None => s.push_str(" - "),
        }

        s.push_str(&format!("{} {}", self.half_move_clock, self.move_number));

        s
    }
}
