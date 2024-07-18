use std::fmt::{Debug, Display};

use super::{BaseMoveType, PromotionPiece};
use crate::game::Square;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Move(u8, u8, BaseMoveType);

impl Move {
    pub fn new(source: u8, dest: u8, move_type: BaseMoveType) -> Move {
        Move(source, dest, move_type)
    }

    pub fn new_sq(source: Square, dest: Square, move_type: BaseMoveType) -> Move {
        Move(source.into(), dest.into(), move_type)
    }

    pub fn source(&self) -> u8 {
        self.0
    }

    pub fn dest(&self) -> u8 {
        self.1
    }

    pub fn source_sq(&self) -> Square {
        Square::from(self.0)
    }

    pub fn dest_sq(&self) -> Square {
        Square::from(self.1)
    }

    pub fn move_type(&self) -> BaseMoveType {
        self.2
    }

    pub fn promotion_piece(&self) -> Option<PromotionPiece> {
        match self.2 {
            BaseMoveType::Promotion(piece) => Some(piece.into()),
            _ => None,
        }
    }

    pub fn null() -> Move {
        Move(std::u8::MAX, std::u8::MAX, BaseMoveType::Normal)
    }

    pub fn is_null(&self) -> bool {
        self.0 == std::u8::MAX && self.1 == std::u8::MAX
    }

    pub fn valid(&self) -> bool {
        (Square::valid(self.0) && Square::valid(self.1)) || self.is_null()
    }

    pub fn as_source_dest(&self) -> String {
        format!(
            "{}{}{}",
            Square::from(self.source()),
            Square::from(self.dest()),
            self.promotion_piece()
                .map(|p| Into::<char>::into(p).to_string())
                .unwrap_or("".to_string())
        )
    }

    pub fn from_source_dest(str: &str) -> Move {
        let source = Square::from(&str[0..2]);
        let dest = Square::from(&str[2..4]);
        let move_type = match str.len() {
            5 => BaseMoveType::Promotion((&str[4..5].chars().next().unwrap()).into()),
            _ => BaseMoveType::Normal,
        };

        Move::new_sq(source, dest, move_type)
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_null() {
            return write!(f, "0000");
        }

        if !self.valid() {
            return write!(f, "->Invalid move<-");
        }

        if self.promotion_piece().is_some() {
            return write!(
                f,
                "{}->{}|{}",
                self.source_sq(),
                self.dest_sq(),
                Into::<char>::into(self.promotion_piece().unwrap())
            );
        }

        write!(f, "{}->{}", self.source_sq(), self.dest_sq())
    }
}

impl Default for Move {
    fn default() -> Self {
        Move::null()
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
