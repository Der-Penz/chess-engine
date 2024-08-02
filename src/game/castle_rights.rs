use thiserror::Error;

use super::{color::Color, square::Square};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastleType {
    KingSide,
    QueenSide,
}

impl std::fmt::Display for CastleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CastleType::KingSide => write!(f, "O-O"),
            CastleType::QueenSide => write!(f, "O-O-O"),
        }
    }
}

impl<T, const N: usize> std::ops::Index<CastleType> for [T; N] {
    type Output = T;

    fn index(&self, index: CastleType) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T, const N: usize> std::ops::IndexMut<CastleType> for [T; N] {
    fn index_mut(&mut self, index: CastleType) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

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

    pub fn to_index(color: &Color, castle_type: &CastleType) -> u8 {
        match (color, castle_type) {
            (Color::White, CastleType::KingSide) => 0,
            (Color::White, CastleType::QueenSide) => 1,
            (Color::Black, CastleType::KingSide) => 2,
            (Color::Black, CastleType::QueenSide) => 3,
        }
    }

    pub fn as_u8(&self) -> u8 {
        self.0
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
        if self.has(&Color::White, &CastleType::KingSide) {
            rights.push('K');
        }
        if self.has(&Color::White, &CastleType::QueenSide) {
            rights.push('Q');
        }
        if self.has(&Color::Black, &CastleType::KingSide) {
            rights.push('k');
        }
        if self.has(&Color::Black, &CastleType::QueenSide) {
            rights.push('q');
        }
        if rights.is_empty() {
            write!(f, "-")
        } else {
            write!(f, "{}", rights)
        }
    }
}

impl std::fmt::Debug for CastleRights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

#[derive(Debug, Error)]
pub enum CastleRightsError {
    #[error("Invalid character {0}")]
    InvalidCharacter(char),
}

impl TryFrom<&str> for CastleRights {
    type Error = CastleRightsError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut rights = CastleRights::default();

        if value == "-" {
            return Ok(rights);
        }

        for c in value.chars() {
            match c {
                'K' => rights.update(&Color::White, &CastleType::KingSide, true),
                'Q' => rights.update(&Color::White, &CastleType::QueenSide, true),
                'k' => rights.update(&Color::Black, &CastleType::KingSide, true),
                'q' => rights.update(&Color::Black, &CastleType::QueenSide, true),
                _ => return Err(CastleRightsError::InvalidCharacter(c)),
            }
        }
        Ok(rights)
    }
}

impl CastleType {
    /// Returns the source square of the rook for the given castle type and color (indexed by CastleRights::to_index)
    const ROOK_SOURCE: [Square; 4] = [Square::H1, Square::A1, Square::H8, Square::A8];
    /// Returns the dest square of the rook for the given castle type and color (indexed by CastleRights::to_index)
    const ROOK_DEST: [Square; 4] = [Square::F1, Square::D1, Square::F8, Square::D8];

    /// Returns the source square of the king for the color
    pub const KING_SOURCE: [Square; 2] = [Square::E1, Square::E8];
    /// Returns the dest square of the king for the given castle type and color (indexed by CastleRights::to_index)
    pub const KING_DEST: [Square; 4] = [Square::G1, Square::C1, Square::G8, Square::C8];

    /// Checks which castle type satisfies the move from and to squares
    pub fn satisfies_castle(from: &Square, to: &Square, color: &Color) -> Option<Self> {
        if *from == CastleType::KING_SOURCE[*color]
            && *to
                == CastleType::KING_DEST
                    [CastleRights::to_index(color, &CastleType::KingSide) as usize]
        {
            return Some(CastleType::KingSide);
        } else if *from == CastleType::KING_SOURCE[*color]
            && *to
                == CastleType::KING_DEST
                    [CastleRights::to_index(&color, &CastleType::QueenSide) as usize]
        {
            return Some(CastleType::QueenSide);
        }
        None
    }

    pub fn get_rook_positions(&self, color: &Color) -> (&Square, &Square) {
        (
            &CastleType::ROOK_SOURCE[CastleRights::to_index(color, &self) as usize],
            &CastleType::ROOK_DEST[CastleRights::to_index(color, &self) as usize],
        )
    }
}
