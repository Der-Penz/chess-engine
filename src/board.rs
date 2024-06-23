use core::panic;
use std::{ any::Any, fmt::Display };

use crate::{
    piece::{ self, Color, Piece, PieceVariation },
    position::{ match_piece, to_board_bit },
};

const PIECES_BOARD: usize = 6;

enum BitBoardOperation {
    SET,
    RESET,
}

pub struct Board {
    black_boards: [u64; 7],
    white_boards: [u64; 7],
    white_can_castled: bool,
    black_can_castled: bool,
    en_passant: u8, //notes the square on which an en passant can happen next move
}

impl Board {
    pub fn new() -> Self {
        let pawn = 0xff00;
        let bishop = 0x24;
        let knight = 0x42;
        let rook = 0x81;
        let queen = 0x8;
        let king = 0x10;
        let all = pawn | knight | bishop | rook | queen | king;
        Board {
            white_boards: [pawn, knight, bishop, rook, queen, king, all],
            black_boards: [
                pawn << 40,
                knight << 56,
                bishop << 56,
                rook << 56,
                queen << 56,
                king << 56,
                all << 48,
            ],
            black_can_castled: true,
            white_can_castled: true,

            en_passant: 0xff,
        }
    }

    /*
        Just for visualization of attack patterns
     */
    pub fn only_pawns(position_white: u64, position_black: u64) -> Self {
        Board {
            white_boards: [position_white, 0, 0, 0, 0, 0, position_white],
            black_boards: [position_black, 0, 0, 0, 0, 0, position_black],
            black_can_castled: true,
            white_can_castled: true,
            en_passant: 0xff,
        }
    }

    fn get_field_color(&self, pos: u8) -> Option<Color> {
        if match_piece(pos, self.black_boards[PIECES_BOARD]) {
            Some(Color::BLACK)
        } else if match_piece(pos, self.white_boards[PIECES_BOARD]) {
            Some(Color::WHITE)
        } else {
            None
        }
    }

    fn get_field_piece_variation(&self, pos: u8) -> Option<PieceVariation> {
        for (piece, index) in PieceVariation::iter() {
            if match_piece(pos, self.black_boards[index] | self.white_boards[index]) {
                return Some(piece);
            }
        }
        None
    }

    pub fn get_piece(&self, pos: u8) -> Option<Piece> {
        let color = self.get_field_color(pos);
        let piece_variation = self.get_field_piece_variation(pos);

        match color {
            Some(color) =>
                Some(
                    Piece(
                        piece_variation.expect("Field must have a piece as it has a color"),
                        color
                    )
                ),
            None => None,
        }
    }

    fn update_bit_board(&mut self, piece: &Piece, pos: u8, op: BitBoardOperation) {
        match piece {
            piece if matches!(piece.1, Color::WHITE) => {
                match op {
                    BitBoardOperation::SET => {
                        self.white_boards[PIECES_BOARD] |= to_board_bit(pos);
                        self.white_boards[piece.0 as usize] |= to_board_bit(pos);
                    }
                    BitBoardOperation::RESET => {
                        self.white_boards[PIECES_BOARD] &= !to_board_bit(pos);
                        self.white_boards[piece.0 as usize] &= !to_board_bit(pos);
                    }
                }
            }
            piece if matches!(piece.1, Color::BLACK) => {
                match op {
                    BitBoardOperation::SET => {
                        self.black_boards[PIECES_BOARD] |= to_board_bit(pos);
                        self.black_boards[piece.1 as usize] |= to_board_bit(pos);
                    }
                    BitBoardOperation::RESET => {
                        self.black_boards[PIECES_BOARD] &= !to_board_bit(pos);
                        self.black_boards[piece.0 as usize] &= !to_board_bit(pos);
                    }
                }
            }
            _ => panic!("Invalid State, Color must be BLACK or WHITE"),
        }
    }

    pub fn move_piece(&mut self, source: u8, dest: u8) -> Option<Piece> {
        let Some(source_piece) = self.get_piece(source) else {
            return None;
        };
        let dest_piece = self.get_piece(dest);

        if let Some(dest_piece) = dest_piece {
            //can't move if the dest square is occupied by a piece of the same color
            if source_piece.1 == dest_piece.1 {
                return None;
            }
        }
        self.update_bit_board(&source_piece, source, BitBoardOperation::RESET);
        self.update_bit_board(&source_piece, dest, BitBoardOperation::SET);
        None
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut repr = String::new();

        repr.push_str(" ");
        for x in 'A'..'I' {
            repr.push_str(&format!(" {x}"));
        }
        repr.push_str("\n");

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
