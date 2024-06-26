use std::fmt::Display;

use num_derive::FromPrimitive;
use num_traits::{ FromPrimitive as FromPrim, Zero };

use super::{ valid_position, to_field_repr, Color, Piece, PieceVariation };

#[derive(Debug)]
pub struct Move(u32);

#[derive(Debug, FromPrimitive)]
pub enum PromotionPiece {
    BISHOP,
    KNIGHT,
    QUEEN,
    ROOK,
}

#[derive(Debug, FromPrimitive)]
pub enum MoveType {
    NORMAL,
    PROMOTION,
    ENPASSANT,
    CASTLING,
}

static SOURCE_OFFSET: u32 = 0;
static DEST_OFFSET: u32 = 7;
static PIECE_OFFSET: u32 = 13;
static COLOR_OFFSET: u32 = 16;
static PROMOTION_PIECE_OFFSET: u32 = 17;
static MOVE_TYPE_OFFSET: u32 = 19;

static SOURCE_MASK: u32 = 0b111111 << SOURCE_OFFSET;
static DEST_MASK: u32 = 0b111111 << DEST_OFFSET;
static PIECE_MASK: u32 = 0b111 << PIECE_OFFSET;
static COLOR_MASK: u32 = 0b1 << COLOR_OFFSET;
static PROMOTION_PIECE_MASK: u32 = 0b11 << PROMOTION_PIECE_OFFSET;
static MOVE_TYPE_MASK: u32 = 0b11 << MOVE_TYPE_OFFSET;

impl Move {
    pub fn new(
        source: u8,
        dest: u8,
        piece: Piece,
        move_type: MoveType,
        promotion_piece: PromotionPiece
    ) -> Self {
        assert!(valid_position(source));
        assert!(valid_position(dest));

        let mut m: u32 = 0x0;
        m = m | ((source as u32) << SOURCE_OFFSET);
        m = m | ((dest as u32) << DEST_OFFSET);
        m = m | ((piece.0 as u32) << PIECE_OFFSET);
        m = m | ((piece.1 as u32) << COLOR_OFFSET);
        m = m | ((promotion_piece as u32) << PROMOTION_PIECE_OFFSET);
        m = m | ((move_type as u32) << MOVE_TYPE_OFFSET);

        Move(m)
    }

    pub fn bits(&self) -> u32 {
        self.0
    }

    pub fn normal(source: u8, dest: u8, piece: Piece) -> Self {
        Move::new(source, dest, piece, MoveType::NORMAL, PromotionPiece::BISHOP)
    }

    pub fn promotion(source: u8, dest: u8, color: Color, promotion_piece: PromotionPiece) -> Self {
        Move::new(
            source,
            dest,
            Piece(PieceVariation::PAWN, color),
            MoveType::PROMOTION,
            promotion_piece
        )
    }

    pub fn en_passant(source: u8, dest: u8, color: Color) -> Self {
        Move::new(
            source,
            dest,
            Piece(PieceVariation::PAWN, color),
            MoveType::ENPASSANT,
            PromotionPiece::BISHOP
        )
    }

    pub fn castle(source: u8, dest: u8, color: Color) -> Self {
        Move::new(
            source,
            dest,
            Piece(PieceVariation::KING, color),
            MoveType::CASTLING,
            PromotionPiece::BISHOP
        )
    }

    pub fn null() -> Self {
        Move(std::u32::MAX)
    }

    pub fn source(&self) -> u8 {
        ((self.0 & SOURCE_MASK) >> SOURCE_OFFSET) as u8
    }

    pub fn dest(&self) -> u8 {
        ((self.0 & DEST_MASK) >> DEST_OFFSET) as u8
    }

    pub fn piece_variation(&self) -> PieceVariation {
        let n = (self.0 & PIECE_MASK) >> PIECE_OFFSET;
        PieceVariation::from_u32(n).unwrap_or(PieceVariation::PAWN)
    }

    pub fn color(&self) -> Color {
        Color::from_u32((self.0 & COLOR_MASK) >> COLOR_OFFSET).expect(
            "Should always be parsable since it has 2 variants"
        )
    }

    pub fn piece(&self) -> Piece {
        Piece(self.piece_variation(), self.color())
    }

    pub fn move_type(&self) -> MoveType {
        MoveType::from_u32((self.0 & MOVE_TYPE_MASK) >> MOVE_TYPE_OFFSET).expect(
            "Should always be parsable since it has two variants"
        )
    }

    pub fn promotion_piece(&self) -> PromotionPiece {
        PromotionPiece::from_u32((self.0 & PROMOTION_PIECE_MASK) >> PROMOTION_PIECE_OFFSET).expect(
            "Should always be parsable since it has two variants"
        )
    }

    pub fn is_null(&self) -> bool {
        self.0.is_zero()
    }

    pub fn valid(&self) -> bool {
        (self.0 & PIECE_MASK) >> PIECE_OFFSET < 7
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.move_type() {
            MoveType::NORMAL =>
                write!(
                    f,
                    "{}->{} {} | Normal",
                    to_field_repr(self.source()),
                    to_field_repr(self.dest()),
                    self.piece()
                ),
            MoveType::PROMOTION =>
                write!(
                    f,
                    "{}->{} {} | Promotion -> {:?}",
                    to_field_repr(self.source()),
                    to_field_repr(self.dest()),
                    self.piece(),
                    self.promotion_piece()
                ),
            MoveType::ENPASSANT =>
                write!(
                    f,
                    "{}->{} {} | En Passant",
                    to_field_repr(self.source()),
                    to_field_repr(self.dest()),
                    self.piece()
                ),
            MoveType::CASTLING =>
                write!(
                    f,
                    "{}->{} {}| Castling",
                    to_field_repr(self.source()),
                    to_field_repr(self.dest()),
                    self.piece()
                ),
        }
    }
}
