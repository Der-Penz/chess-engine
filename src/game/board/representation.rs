use std::fmt::Display;

use crate::game::{ Color, Piece, PieceVariation, Square };

use super::{ BitBoardOperation, Board };

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut repr = String::new();

        repr.push_str(" ");
        for x in 'A'..'I' {
            repr.push_str(&format!(" {x}"));
        }
        if self.color_to_move == Color::WHITE {
            repr.push_str("  ⬤\n");
        } else {
            repr.push_str("  ◯\n");
        }

        for y in 0..8 {
            let y = 7 - y;
            repr.push_str(&format!("{}", y + 1));
            for x in 0..8 {
                let piece = self.get_piece(x + y * 8);
                match piece {
                    Some(piece) => repr.push_str(&format!(" {}", piece)),
                    None => repr.push_str(&format!(" {}", " ")),
                }
            }
            repr.push_str(&format!("  {}\n", y + 1));
        }

        repr.push_str(" ");
        for x in 'A'..'I' {
            repr.push_str(&format!(" {x}"));
        }
        write!(f, "{}", repr)
    }
}

#[derive(Debug)]
pub enum FENError {
    ParsingError,
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
                return Err(FENError::ParsingError);
            }
            match char {
                '/' => {
                    row -= 1;
                    col = 0;
                }
                number if number.is_digit(10) => {
                    col += number.to_digit(10).expect("Must be a digit") as i8;
                }
                'r' | 'n' | 'b' | 'q' | 'k' | 'p' => {
                    let piece = Piece(PieceVariation::from(char), Color::BLACK);
                    board.update_bit_board(&piece, (row * 8 + col) as u8, BitBoardOperation::SET);
                    col += 1;
                }
                'R' | 'N' | 'B' | 'Q' | 'K' | 'P' => {
                    let piece = Piece(PieceVariation::from(char), Color::WHITE);
                    board.update_bit_board(&piece, (row * 8 + col) as u8, BitBoardOperation::SET);
                    col += 1;
                }
                _ => {
                    return Err(FENError::ParsingError);
                }
            }
        }

        let fen_group = splits.next().ok_or(FENError::MissingGroup)?;

        match fen_group {
            "w" => board.set_color_to_move(Color::WHITE),
            "b" => board.set_color_to_move(Color::BLACK),
            _ => {
                return Err(FENError::ParsingError);
            }
        }

        let fen_group = splits.next().ok_or(FENError::MissingGroup)?;

        for char in fen_group.chars() {
            match char {
                'K' => board.set_castle(Color::WHITE, true, true),
                'Q' => board.set_castle(Color::WHITE, true, false),
                'k' => board.set_castle(Color::BLACK, true, true),
                'q' => board.set_castle(Color::BLACK, true, false),
                '-' => (),
                _ => {
                    return Err(FENError::ParsingError);
                }
            }
        }

        let fen_group = splits.next().ok_or(FENError::MissingGroup)?;

        match fen_group {
            "-" => board.set_en_passant(0xff),
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

                board.set_en_passant(en_passant);
            }
        }

        //TODO: Implement half move clock and move number
        Ok(board)
    }
}
