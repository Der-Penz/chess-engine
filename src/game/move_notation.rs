use thiserror::Error;

use super::{castle_rights::CastleType, piece_type::PieceType, square::Square, Board};

#[derive(Copy, Clone, PartialEq, Eq)]
/// A space-efficient representation of a move in a chess game.
/// The u16 value is split into 3 parts:
/// - The source square (6 bits)
/// - The destination square (6 bits)
/// - The move flag (4 bits)
pub struct Move(u16);

const SOURCE_SQUARE_MASK: u16 = 0b0000_0000_0000_0000_0000_0000_0011_1111;
const DEST_SQUARE_MASK: u16 = 0b0000_0000_0000_0000_0000_1111_1100_0000;
const FLAGS_MASK: u16 = 0b0000_0000_0000_0000_1111_0000_0000_0000;

const SOURCE_SQUARE_OFFSET: u16 = 0;
const DEST_SQUARE_OFFSET: u16 = 6;
const FLAGS_OFFSET: u16 = 12;

impl std::default::Default for Move {
    fn default() -> Self {
        Move(0)
    }
}

impl Move {
    pub fn new(source: Square, dest: Square, flag: MoveFlag) -> Self {
        let mut first_move_value = 0u16;
        first_move_value |= (source.square_value() as u16) << SOURCE_SQUARE_OFFSET;
        first_move_value |= (dest.square_value() as u16) << DEST_SQUARE_OFFSET;
        first_move_value |= (flag as u16) << FLAGS_OFFSET;

        Move(first_move_value)
    }

    pub fn null() -> Self {
        Move(0)
    }

    pub fn is_null(&self) -> bool {
        self.0 == 0
    }

    pub fn source(&self) -> Square {
        Square::new(((self.0 & SOURCE_SQUARE_MASK) >> SOURCE_SQUARE_OFFSET) as u8)
    }

    pub fn dest(&self) -> Square {
        Square::new(((self.0 & DEST_SQUARE_MASK) >> DEST_SQUARE_OFFSET) as u8)
    }

    pub fn flag(&self) -> MoveFlag {
        MoveFlag::from(((self.0 & FLAGS_MASK) >> FLAGS_OFFSET) as u8)
    }

    /// Convert the move to UCI notation. This conversion is lossy, as the move will not contain more information than the source, destination and Promotion or not.
    pub fn as_uci_notation(&self) -> String {
        if self.is_null() {
            return String::from("0000");
        }

        format!(
            "{}{}{}",
            self.source(),
            self.dest(),
            self.flag()
                .promotion_type()
                .map_or("".to_string(), |piece| piece.to_string())
        )
    }

    /// Parse a move from UCI notation.
    /// Promotion can be specified by =(Type) or (Type) at the end of the move. Example: e7e8q or e7e8=q
    /// Null move can be specified by 0000 or 00-00
    pub fn from_uci_notation(uci: &str, board: &Board) -> Result<Self, UCIMoveParseError> {
        if uci.len() < 4 || uci.len() > 6 {
            return Err(UCIMoveParseError::InvalidLength);
        }

        if uci == "0000" || uci == "00-00" {
            return Ok(Move::null());
        }

        let source =
            Square::try_from(&uci[0..2]).map_err(|_| UCIMoveParseError::InvalidSourceSquare)?;
        let dest =
            Square::try_from(&uci[2..4]).map_err(|_| UCIMoveParseError::InvalidDestSquare)?;
        let moved_piece_type = board
            .get_sq_piece(source)
            .ok_or(UCIMoveParseError::EmptySourceSquare)?
            .ptype();

        let move_flag = match moved_piece_type {
            PieceType::Pawn => {
                if uci.len() > 4 {
                    match &uci[5..=6] {
                        "n" | "=n" => MoveFlag::KnightPromotion,
                        "b" | "=b" => MoveFlag::BishopPromotion,
                        "r" | "=r" => MoveFlag::RookPromotion,
                        "q" | "=q" => MoveFlag::QueenPromotion,
                        _ => return Err(UCIMoveParseError::InvalidPromotionFlag),
                    }
                } else if board.cur_state().en_passant.is_some_and(|sq| sq == dest) {
                    MoveFlag::EnPassant
                } else if source.rank().abs_diff(dest.rank()) == 2 {
                    MoveFlag::DoublePawnPush
                } else {
                    MoveFlag::Normal
                }
            }
            PieceType::King => CastleType::satisfies_castle(source, dest, board.side_to_move())
                .map_or(MoveFlag::Normal, |castle| match castle {
                    CastleType::KingSide => MoveFlag::KingSideCastle,
                    CastleType::QueenSide => MoveFlag::QueenSideCastle,
                }),
            _ => MoveFlag::Normal,
        };

        return Ok(Move::new(source, dest, move_flag));
    }
}

#[derive(Error, Debug)]
pub enum UCIMoveParseError {
    #[error("Invalid length, must be 4 or 5 characters")]
    InvalidLength,
    #[error("Invalid source square")]
    InvalidSourceSquare,
    #[error("Invalid destination square")]
    InvalidDestSquare,
    #[error("Invalid flag for promotion")]
    InvalidPromotionFlag,
    #[error("Cannot parse move with an empty source square")]
    EmptySourceSquare,
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_null() {
            return write!(f, "0000");
        }

        write!(f, "{}{} {:?}", self.source(), self.dest(), self.flag())
    }
}

impl std::fmt::Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MoveFlag {
    Null = 0,
    Normal = 1,          // 0b0001
    KingSideCastle = 2,  // 0b0010
    QueenSideCastle = 3, // 0b0011
    DoublePawnPush = 4,  // 0b0100
    EnPassant = 5,       // 0b0101
    KnightPromotion = 8, // 0b1000
    BishopPromotion = 9, // 0b1001
    RookPromotion = 10,  // 0b1010
    QueenPromotion = 11, // 0b1011
}

impl std::default::Default for MoveFlag {
    fn default() -> Self {
        MoveFlag::Null
    }
}

impl From<u8> for MoveFlag {
    fn from(value: u8) -> Self {
        match value {
            0 => MoveFlag::Null,
            1 => MoveFlag::Normal,
            2 => MoveFlag::KingSideCastle,
            3 => MoveFlag::QueenSideCastle,
            4 => MoveFlag::DoublePawnPush,
            5 => MoveFlag::EnPassant,
            8 => MoveFlag::KnightPromotion,
            9 => MoveFlag::BishopPromotion,
            10 => MoveFlag::RookPromotion,
            11 => MoveFlag::QueenPromotion,
            _ => MoveFlag::Null,
        }
    }
}

impl From<MoveFlag> for u8 {
    fn from(flag: MoveFlag) -> u8 {
        flag as u8
    }
}

impl MoveFlag {
    pub fn is_promotion(&self) -> bool {
        (*self as u8) & 0b1000 != 0
    }

    pub fn is_castle(&self) -> bool {
        (*self as u8) & 0b1110 == 0b0010
    }

    pub fn is_en_passant(&self) -> bool {
        *self == MoveFlag::EnPassant
    }

    pub fn is_double_pawn_push(&self) -> bool {
        *self == MoveFlag::DoublePawnPush
    }

    pub fn is_normal(&self) -> bool {
        *self == MoveFlag::Normal
    }

    pub fn promotion_type(&self) -> Option<PieceType> {
        match self {
            MoveFlag::KnightPromotion => Some(PieceType::Knight),
            MoveFlag::BishopPromotion => Some(PieceType::Bishop),
            MoveFlag::RookPromotion => Some(PieceType::Rook),
            MoveFlag::QueenPromotion => Some(PieceType::Queen),
            _ => None,
        }
    }

    pub fn castle_side(&self) -> Option<CastleType> {
        match self {
            MoveFlag::KingSideCastle => Some(CastleType::KingSide),
            MoveFlag::QueenSideCastle => Some(CastleType::QueenSide),
            _ => None,
        }
    }

    pub const PAWN_PROMOTION_FLAGS: [MoveFlag; 4] = [
        MoveFlag::KnightPromotion,
        MoveFlag::BishopPromotion,
        MoveFlag::RookPromotion,
        MoveFlag::QueenPromotion,
    ];
}
