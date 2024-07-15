use std::fmt;

use super::{ Color, PieceVariation };

#[derive(Debug, Copy, Clone)]
pub struct Piece(pub PieceVariation, pub Color);

impl Piece {
    pub fn black_rook() -> Piece {
        Piece(PieceVariation::ROOK, Color::BLACK)
    }
    pub fn black_bishop() -> Piece {
        Piece(PieceVariation::BISHOP, Color::BLACK)
    }
    pub fn black_queen() -> Piece {
        Piece(PieceVariation::QUEEN, Color::BLACK)
    }
    pub fn black_king() -> Piece {
        Piece(PieceVariation::KING, Color::BLACK)
    }
    pub fn black_knight() -> Piece {
        Piece(PieceVariation::KNIGHT, Color::BLACK)
    }
    pub fn black_pawn() -> Piece {
        Piece(PieceVariation::PAWN, Color::BLACK)
    }
    pub fn white_rook() -> Piece {
        Piece(PieceVariation::ROOK, Color::WHITE)
    }
    pub fn white_bishop() -> Piece {
        Piece(PieceVariation::BISHOP, Color::WHITE)
    }
    pub fn white_queen() -> Piece {
        Piece(PieceVariation::QUEEN, Color::WHITE)
    }
    pub fn white_king() -> Piece {
        Piece(PieceVariation::KING, Color::WHITE)
    }
    pub fn white_knight() -> Piece {
        Piece(PieceVariation::KNIGHT, Color::WHITE)
    }
    pub fn white_pawn() -> Piece {
        Piece(PieceVariation::PAWN, Color::WHITE)
    }

    pub fn color_matches(&self, other: Piece) -> bool {
        self.1 == other.1
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let black = matches!(self.1, Color::BLACK);
        let repr = match self.0 {
            PieceVariation::PAWN if black => "♙",
            PieceVariation::BISHOP if black => "♗",
            PieceVariation::KNIGHT if black => "♘",
            PieceVariation::ROOK if black => "♖",
            PieceVariation::KING if black => "♔",
            PieceVariation::QUEEN if black => "♕",
            PieceVariation::PAWN => "♟︎",
            PieceVariation::BISHOP => "♝",
            PieceVariation::KNIGHT => "♞",
            PieceVariation::ROOK => "♜",
            PieceVariation::KING => "♚",
            PieceVariation::QUEEN => "♛",
        };
        write!(f, "{}", repr)
    }
}

impl Default for Piece{
    fn default() -> Self {
        Self(Default::default(), Default::default())
    }
}