use crate::game::PieceVariation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromotionPiece {
    Knight,
    Bishop,
    Rook,
    Queen,
}

impl From<&char> for PromotionPiece {
    fn from(value: &char) -> Self {
        match value.to_ascii_lowercase() {
            'n' => PromotionPiece::Knight,
            'b' => PromotionPiece::Bishop,
            'r' => PromotionPiece::Rook,
            'q' => PromotionPiece::Queen,
            _ => panic!("Invalid promotion piece: {}", value),
        }
    }
}

impl From<PromotionPiece> for char {
    fn from(value: PromotionPiece) -> Self {
        match value {
            PromotionPiece::Knight => 'n',
            PromotionPiece::Bishop => 'b',
            PromotionPiece::Rook => 'r',
            PromotionPiece::Queen => 'q',
        }
    }
}

impl From<PromotionPiece> for PieceVariation{
    fn from(value: PromotionPiece) -> Self {
        match value {
            PromotionPiece::Knight => PieceVariation::KNIGHT,
            PromotionPiece::Bishop => PieceVariation::BISHOP,
            PromotionPiece::Rook => PieceVariation::ROOK,
            PromotionPiece::Queen => PieceVariation::QUEEN,
        }
    }
}

impl std::fmt::Display for PromotionPiece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr = match self {
            PromotionPiece::Knight => "Knight",
            PromotionPiece::Bishop => "Bishop",
            PromotionPiece::Rook => "Rook",
            PromotionPiece::Queen => "Queen",
        };
        write!(f, "{}", repr)
    }
}
