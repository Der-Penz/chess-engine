use core::panic;

use crate::game::Square;

use super::{ match_piece, Color, Piece, PieceVariation };

mod move_generation;
mod representation;

const PIECES_BOARD: usize = 6;

enum BitBoardOperation {
    SET,
    RESET,
}

pub struct Board {
    black_boards: [u64; 7],
    white_boards: [u64; 7],
    color_to_move: Color,
    white_castle: (bool, bool), //(king side, queen side)
    black_castle: (bool, bool), // (king side, queen side)
    en_passant: u8, //notes the square behind the pawn that can be captured en passant.
                    // a value over 63 means no en passant
                    // e.g if pawn moves from F2 to F4, F3 is the en passant square
}

impl Board {
    pub fn empty() -> Self {
        Board {
            white_boards: [0, 0, 0, 0, 0, 0, 0],
            black_boards: [0, 0, 0, 0, 0, 0, 0],
            color_to_move: Color::WHITE,
            white_castle: (true, true),
            black_castle: (true, true),
            en_passant: 0xff,
        }
    }

    pub fn base() -> Self {
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq e4 0 1").expect(
            "Invalid base FEN String"
        )
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
            piece if piece.1 == Color::WHITE => {
                match op {
                    BitBoardOperation::SET => {
                        self.white_boards[PIECES_BOARD] |= Square::to_board_bit(pos);
                        self.white_boards[piece.0] |= Square::to_board_bit(pos);
                    }
                    BitBoardOperation::RESET => {
                        self.white_boards[PIECES_BOARD] &= !Square::to_board_bit(pos);
                        self.white_boards[piece.0] &= !Square::to_board_bit(pos);
                    }
                }
            }
            piece if piece.1 == Color::BLACK => {
                match op {
                    BitBoardOperation::SET => {
                        self.black_boards[PIECES_BOARD] |= Square::to_board_bit(pos);
                        self.black_boards[piece.0] |= Square::to_board_bit(pos);
                    }
                    BitBoardOperation::RESET => {
                        self.black_boards[PIECES_BOARD] &= !Square::to_board_bit(pos);
                        self.black_boards[piece.0] &= !Square::to_board_bit(pos);
                    }
                }
            }
            _ => panic!("Invalid State, Color must be BLACK or WHITE"),
        }
    }

    pub fn move_piece(&mut self, source: u8, dest: u8) -> Option<Piece> {
        let source_piece = self.get_piece(source)?;
        let dest_piece = self.get_piece(dest);

        self.update_bit_board(&source_piece, source, BitBoardOperation::RESET);
        self.update_bit_board(&source_piece, dest, BitBoardOperation::SET);

        if let Some(dest_piece) = dest_piece {
            println!("To {}", dest_piece);
            self.update_bit_board(&dest_piece, dest, BitBoardOperation::RESET);
        }
        println!("Move: {} from {} to {}", source_piece, Square::from(source), Square::from(dest));
        None
    }

    pub fn get_boards(&self, color: &Color) -> [u64; 7] {
        match color {
            Color::WHITE => self.white_boards,
            Color::BLACK => self.black_boards,
        }
    }

    pub fn get_pieces_board(&self, color: &Color) -> u64 {
        self.get_boards(color)[6]
    }

    fn set_en_passant(&mut self, square: u8) {
        self.en_passant = square;
    }

    fn set_color_to_move(&mut self, color: Color) {
        self.color_to_move = color;
    }

    fn set_castle(&mut self, color: Color, value: bool, king_side: bool) {
        match color {
            Color::WHITE => {
                if king_side {
                    self.white_castle.0 = value;
                } else {
                    self.white_castle.1 = value;
                }
            }
            Color::BLACK => {
                if king_side {
                    self.black_castle.0 = value;
                } else {
                    self.black_castle.1 = value;
                }
            }
        }
    }
}
