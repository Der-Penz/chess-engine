use crate::game::Square;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastleType {
    KingSide,
    QueenSide,
}

impl CastleType {
    pub fn is_king_side(&self) -> bool {
        match self {
            CastleType::KingSide => true,
            _ => false,
        }
    }

    pub fn is_queen_side(&self) -> bool {
        match self {
            CastleType::QueenSide => true,
            _ => false,
        }
    }

    pub fn matches_king_side(from: Square, to: Square) -> bool {
        (from == Square::E1 && to == Square::G1) || (from == Square::E8 && to == Square::G8)
    }

    pub fn matches_queen_side(from: Square, to: Square) -> bool {
        (from == Square::E1 && to == Square::C1) || (from == Square::E8 && to == Square::C8)
    }

    pub fn matches_castle(from: Square, to: Square) -> Option<Self> {
        if Self::matches_king_side(from, to) {
            Some(CastleType::KingSide)
        } else if Self::matches_queen_side(from, to) {
            Some(CastleType::QueenSide)
        } else {
            None
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