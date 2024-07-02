use std::fmt::Display;

use num_derive::FromPrimitive;
use num_traits::{ FromPrimitive as FromPrim, Zero };

use super::{ Color, Piece, PieceVariation, Square };

#[derive(Debug)]
pub struct Move(u32);

#[derive(Debug, FromPrimitive)]
pub enum PromotionPiece {
    BISHOP,
    KNIGHT,
    QUEEN,
    ROOK,
}

#[derive(Debug, FromPrimitive, PartialEq, Eq)]
pub enum MoveType {
    NORMAL,
    PROMOTION,
    ENPASSANT,
    CASTLING,
}

static SOURCE_OFFSET: u32 = 0;
static DEST_OFFSET: u32 = 6;
static PIECE_OFFSET: u32 = 12;
static COLOR_OFFSET: u32 = 15;
static PROMOTION_PIECE_OFFSET: u32 = 16;
static MOVE_TYPE_OFFSET: u32 = 18;
static CAPTURED_PICE_OFFSET: u32 = 20;

static SOURCE_MASK: u32 = 0b111111 << SOURCE_OFFSET;
static DEST_MASK: u32 = 0b111111 << DEST_OFFSET;
static PIECE_MASK: u32 = 0b111 << PIECE_OFFSET;
static COLOR_MASK: u32 = 0b1 << COLOR_OFFSET;
static PROMOTION_PIECE_MASK: u32 = 0b11 << PROMOTION_PIECE_OFFSET;
static MOVE_TYPE_MASK: u32 = 0b11 << MOVE_TYPE_OFFSET;
static CAPTURED_PICE_MASK: u32 = 0b111 << CAPTURED_PICE_OFFSET;

impl Move {
    pub fn new(
        source: u8,
        dest: u8,
        piece: Piece,
        move_type: MoveType,
        promotion_piece: PromotionPiece,
        captured_piece: Option<PieceVariation>
    ) -> Self {
        assert!(Square::valid(source));
        assert!(Square::valid(dest));

        let mut m: u32 = 0x0;
        m = m | ((source as u32) << SOURCE_OFFSET);
        m = m | ((dest as u32) << DEST_OFFSET);
        m = m | ((piece.0 as u32) << PIECE_OFFSET);
        m = m | ((piece.1 as u32) << COLOR_OFFSET);
        m = m | ((promotion_piece as u32) << PROMOTION_PIECE_OFFSET);
        m = m | ((move_type as u32) << MOVE_TYPE_OFFSET);
        if captured_piece.is_some() {
            m = m | ((captured_piece.unwrap() as u32) << CAPTURED_PICE_OFFSET);
        } else {
            m = m | (0b111 << CAPTURED_PICE_OFFSET);
        }

        Move(m)
    }

    pub fn bits(&self) -> u32 {
        self.0
    }

    pub fn normal(source: u8, dest: u8, piece: Piece, captured: Option<PieceVariation>) -> Self {
        Move::new(source, dest, piece, MoveType::NORMAL, PromotionPiece::BISHOP, captured)
    }

    pub fn promotion(
        source: u8,
        dest: u8,
        color: Color,
        promotion_piece: PromotionPiece,
        captured: Option<PieceVariation>
    ) -> Self {
        Move::new(
            source,
            dest,
            Piece(PieceVariation::PAWN, color),
            MoveType::PROMOTION,
            promotion_piece,
            captured
        )
    }

    pub fn en_passant(source: u8, dest: u8, color: Color) -> Self {
        Move::new(
            source,
            dest,
            Piece(PieceVariation::PAWN, color),
            MoveType::ENPASSANT,
            PromotionPiece::BISHOP,
            Some(PieceVariation::PAWN)
        )
    }

    pub fn castle(source: u8, dest: u8, color: Color) -> Self {
        Move::new(
            source,
            dest,
            Piece(PieceVariation::KING, color),
            MoveType::CASTLING,
            PromotionPiece::BISHOP,
            None
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

    pub fn captured_piece(&self) -> Option<PieceVariation> {
        let n = (self.0 & CAPTURED_PICE_MASK) >> CAPTURED_PICE_OFFSET;
        PieceVariation::from_u32(n)
    }

    pub fn is_capture(&self) -> bool {
        self.captured_piece().is_some()
    }

    pub fn castle_kingside(&self) -> bool {
        self.move_type() == MoveType::CASTLING && self.dest() == Square::G1.into()
    }

    pub fn castle_queenside(&self) -> bool {
        self.move_type() == MoveType::CASTLING && self.dest() == Square::B1.into()
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
        write!(
            f,
            "{}: {}->{} | ",
            self.piece(),
            Square::from(self.source()),
            Square::from(self.dest())
        )?;
        self.captured_piece().inspect(|c| {
            write!(f, "Captured: {}", Piece(*c, self.color().opposite())).ok();
        });
        match self.move_type() {
            MoveType::NORMAL => write!(f, "Normal"),
            MoveType::PROMOTION => write!(f, "Promotion -> {:?}", self.promotion_piece()),
            MoveType::ENPASSANT => write!(f, "En Passant"),
            MoveType::CASTLING =>
                write!(f, "Castling {}", if self.castle_kingside() { "O-O" } else { "O-O-O" }),
        }
    }
}
