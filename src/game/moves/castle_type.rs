use crate::game::{Color, Square};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastleType {
    KingSide,
    QueenSide,
}

impl CastleType {
    pub fn satisfies_king_side(from: &Square, to: &Square, color: &Color) -> bool {
        match color {
            Color::WHITE => *from == Square::E1 && *to == Square::G1,
            Color::BLACK => *from == Square::E8 && *to == Square::G8,
        }
    }

    pub fn satisfies_queen_side(from: &Square, to: &Square, color: &Color) -> bool {
        match color {
            Color::WHITE => *from == Square::E1 && *to == Square::C1,
            Color::BLACK => *from == Square::E8 && *to == Square::C8,
        }
    }

    pub fn satisfies_castle(from: &Square, to: &Square, color: &Color) -> Option<Self> {
        if CastleType::satisfies_king_side(from, to, color) {
            Some(CastleType::KingSide)
        } else if CastleType::satisfies_queen_side(from, to, color) {
            Some(CastleType::QueenSide)
        } else {
            None
        }
    }

    pub fn get_rook_source(&self, color: &Color) -> Square {
        match self {
            CastleType::KingSide => match color {
                Color::WHITE => Square::H1,
                Color::BLACK => Square::H8,
            },
            CastleType::QueenSide => match color {
                Color::WHITE => Square::A1,
                Color::BLACK => Square::A8,
            },
        }
    }

    pub fn get_rook_dest(&self, color: &Color) -> Square {
        match self {
            CastleType::KingSide => match color {
                Color::WHITE => Square::F1,
                Color::BLACK => Square::F8,
            },
            CastleType::QueenSide => match color {
                Color::WHITE => Square::D1,
                Color::BLACK => Square::D8,
            },
        }
    }
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
