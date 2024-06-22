use std::fmt::Display;

use crate::{ piece::{ Color, Piece, PieceVariation }, position::match_piece };

pub struct Board {
    black_pawn: u64,
    black_knight: u64,
    black_rook: u64,
    black_bishop: u64,
    black_queen: u64,
    black_king: u64,

    black_pieces: u64,

    white_pawn: u64,
    white_knight: u64,
    white_rook: u64,
    white_bishop: u64,
    white_queen: u64,
    white_king: u64,

    white_pieces: u64,

    white_can_castled: bool,
    black_can_castled: bool,
    en_passant: u8, //notes the square on which an en passant can happen next move
}

impl Board {
    pub fn new() -> Self {
        Board {
            black_pawn: 0xff000000000000,
            black_bishop: 0x2400000000000000,
            black_knight: 0x4200000000000000,
            black_rook: 0x8100000000000000,
            black_queen: 0x800000000000000,
            black_king: 0x1000000000000000,
            white_pawn: 0xff00,
            white_bishop: 0x24,
            white_knight: 0x42,
            white_rook: 0x81,
            white_queen: 0x8,
            white_king: 0x10,

            black_pieces: 0xffff000000000000,
            white_pieces: 0xffff,

            black_can_castled: true,
            white_can_castled: true,

            en_passant: 0xff,
        }
    }

    fn get_field_color(&self, pos: u8) -> Option<Color> {
        if match_piece(pos, self.black_pieces) {
            Some(Color::BLACK)
        } else if match_piece(pos, self.white_pieces) {
            Some(Color::WHITE)
        } else {
            None
        }
    }

    fn get_field_piece_variation(&self, pos: u8) -> Option<PieceVariation> {
        if match_piece(pos, self.black_king | self.white_king) {
            Some(PieceVariation::KING)
        } else if match_piece(pos, self.black_queen | self.white_queen) {
            Some(PieceVariation::QUEEN)
        } else if match_piece(pos, self.black_rook | self.white_rook) {
            Some(PieceVariation::ROOK)
        } else if match_piece(pos, self.black_bishop | self.white_bishop) {
            Some(PieceVariation::BISHOP)
        } else if match_piece(pos, self.black_knight | self.white_knight) {
            Some(PieceVariation::KNIGHT)
        } else if match_piece(pos, self.black_pawn | self.white_pawn) {
            Some(PieceVariation::PAWN)
        } else {
            None
        }
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
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut repr = String::new();

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

        write!(f, "{}", repr)
    }
}
