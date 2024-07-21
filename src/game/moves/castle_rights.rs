use crate::game::Color;

use super::CastleType;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CastleRights(u8);

impl CastleRights {
    pub fn update(&mut self, color: &Color, castle_type: &CastleType, value: bool) {
        let index = Self::to_index(color, castle_type);
        if value {
            self.0 |= 1 << index;
        } else {
            self.0 &= !(1 << index);
        }
    }

    pub fn has(&self, color: &Color, castle_type: &CastleType) -> bool {
        self.0 & (1 << Self::to_index(color, castle_type)) != 0
    }

    pub fn has_any(&self, color: &Color) -> bool {
        self.has(color, &CastleType::KingSide) || self.has(color, &CastleType::QueenSide)
    }

    fn to_index(color: &Color, castle_type: &CastleType) -> u8 {
        match (color, castle_type) {
            (Color::WHITE, CastleType::KingSide) => 0,
            (Color::WHITE, CastleType::QueenSide) => 1,
            (Color::BLACK, CastleType::KingSide) => 2,
            (Color::BLACK, CastleType::QueenSide) => 3,
        }
    }
}

impl Default for CastleRights {
    fn default() -> Self {
        CastleRights(0)
    }
}

impl std::fmt::Display for CastleRights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rights = String::new();
        if self.has(&Color::WHITE, &CastleType::KingSide) {
            rights.push('K');
        }
        if self.has(&Color::WHITE, &CastleType::QueenSide) {
            rights.push('Q');
        }
        if self.has(&Color::BLACK, &CastleType::KingSide) {
            rights.push('k');
        }
        if self.has(&Color::BLACK, &CastleType::QueenSide) {
            rights.push('q');
        }
        if rights.is_empty() {
            write!(f, "-")
        } else {
            write!(f, "{}", rights)
        }
    }
}
