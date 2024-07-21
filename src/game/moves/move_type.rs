use super::{castle_type::CastleType, promotion_piece::PromotionPiece};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BaseMoveType {
    Normal,
    Promotion(PromotionPiece),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveType {
    Normal,
    Promotion(PromotionPiece),
    EnPassant,
    Castling(CastleType),
}

impl MoveType {
    pub fn is_castling(&self) -> bool {
        match self {
            MoveType::Castling(_) => true,
            _ => false,
        }
    }

    pub fn castle_type(&self) -> Option<CastleType> {
        match self {
            MoveType::Castling(castle_type) => Some(*castle_type),
            _ => None,
        }
    }

    pub fn is_promotion(&self) -> bool {
        match self {
            MoveType::Promotion(_) => true,
            _ => false,
        }
    }

    pub fn promotion_piece(&self) -> Option<PromotionPiece> {
        match self {
            MoveType::Promotion(promotion_piece) => Some(*promotion_piece),
            _ => None,
        }
    }

    pub fn is_en_passant(&self) -> bool {
        match self {
            MoveType::EnPassant => true,
            _ => false,
        }
    }

    pub fn is_normal(&self) -> bool {
        match self {
            MoveType::Normal => true,
            _ => false,
        }
    }
}

impl From<MoveType> for BaseMoveType {
    fn from(m: MoveType) -> Self {
        match m {
            MoveType::Promotion(promotion_piece) => BaseMoveType::Promotion(promotion_piece),
            _ => BaseMoveType::Normal,
        }
    }
}

impl From<BaseMoveType> for MoveType {
    fn from(m: BaseMoveType) -> Self {
        match m {
            BaseMoveType::Promotion(promotion_piece) => MoveType::Promotion(promotion_piece),
            BaseMoveType::Normal => MoveType::Normal,
        }
    }
}

impl Default for MoveType {
    fn default() -> Self {
        MoveType::Normal
    }
}

impl Default for BaseMoveType {
    fn default() -> Self {
        BaseMoveType::Normal
    }
}

impl std::fmt::Display for MoveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoveType::Normal => write!(f, "Normal"),
            MoveType::Promotion(promotion_piece) => write!(f, "Promotion({})", promotion_piece),
            MoveType::EnPassant => write!(f, "EnPassant"),
            MoveType::Castling(castle_type) => write!(f, "Castling({})", castle_type),
        }
    }
}
