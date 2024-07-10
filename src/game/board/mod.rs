use core::panic;

use log::info;

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
    half_move_clock: usize, //number of half moves since the last capture or pawn advance
    move_number: usize,
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
            half_move_clock: 0,
            move_number: 1,
        }
    }

    pub fn base() -> Self {
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").expect(
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

    pub fn get_piece(&self, square: u8) -> Option<Piece> {
        let color = self.get_field_color(square);
        let piece_variation = self.get_field_piece_variation(square);

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
            info!("To {}", dest_piece);
            self.update_bit_board(&dest_piece, dest, BitBoardOperation::RESET);
        }
        info!("Move: {} from {} to {}", source_piece, Square::from(source), Square::from(dest));
        None
    }

    pub fn get_bbs_color(&self, color: &Color) -> [u64; 7] {
        match color {
            Color::WHITE => self.white_boards,
            Color::BLACK => self.black_boards,
        }
    }

    pub fn get_bb_color_occupied(&self, color: &Color) -> u64 {
        self.get_bbs_color(color)[6]
    }

    pub fn get_bb_for(&self, piece: &Piece) -> u64 {
        self.get_bbs_color(&piece.1)[piece.0]
    }

    pub fn get_bb_all_occupied(&self) -> u64 {
        self.white_boards[6] | self.black_boards[6]
    }
}
