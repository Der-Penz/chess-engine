use std::{ fmt::Display, io::ErrorKind };

use itertools::PeekingNext;
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

    /// Converts a move to long algebraic notation
    pub fn to_long_algebraic(&self) -> Option<String> {
        if !self.valid() {
            return None;
        }

        if self.is_null() {
            return Some("0000".to_string());
        }

        let mut algebraic = String::new();
        let piece = match self.piece_variation() {
            PieceVariation::KNIGHT => "n",
            PieceVariation::BISHOP => "b",
            PieceVariation::ROOK => "r",
            PieceVariation::QUEEN => "q",
            PieceVariation::KING => "k",
            PieceVariation::PAWN => "",
        };
        match self.color() {
            Color::WHITE => algebraic.push_str(&piece.to_ascii_uppercase()),
            Color::BLACK => algebraic.push_str(&piece),
        }

        algebraic.push_str(&Square::from(self.source()).to_string().to_ascii_lowercase());

        if self.is_capture() {
            algebraic.push('x');
        }

        algebraic.push_str(&Square::from(self.dest()).to_string().to_ascii_lowercase());

        Some(algebraic)
    }

    pub fn from_long_algebraic(al: &str) -> Result<Self, ()> {
        if al == "0000" {
            return Ok(Move::null());
        }
        let chars = &mut al.chars().peekable();
        let char = chars.peek();
        let mut piece = match char {
            Some('N') => Piece::white_knight(),
            Some('n') => Piece::black_knight(),
            Some('B') => Piece::white_bishop(),
            Some('b') => Piece::black_bishop(),
            Some('R') => Piece::white_rook(),
            Some('r') => Piece::black_rook(),
            Some('Q') => Piece::white_queen(),
            Some('q') => Piece::black_queen(),
            Some('K') => Piece::white_king(),
            Some('k') => Piece::black_king(),
            Some(_) => Piece::white_pawn(),
            None => {
                return Err(());
            }
        };

        if piece.0 != PieceVariation::PAWN {
            chars.next();
        }

        let mut source = String::new();
        source.push(chars.next().unwrap());
        source.push(chars.next().unwrap());
        let source = Square::from(source);

        let capture = match chars.peek() {
            Some(&'x') => {
                chars.next();
                true
            }
            _ => false,
        };

        let mut dest = String::new();
        dest.push(chars.next().unwrap());
        dest.push(chars.next().unwrap());
        let dest = Square::from(dest);

        if piece.0 == PieceVariation::PAWN && source.rank() > dest.rank() {
            piece = Piece::black_pawn();
        }

        if chars.peek().is_some() {
            let promotion = match chars.next().unwrap().to_ascii_lowercase() {
                'n' => PromotionPiece::KNIGHT,
                'b' => PromotionPiece::BISHOP,
                'r' => PromotionPiece::ROOK,
                'q' => PromotionPiece::QUEEN,
                _ => {
                    return Err(());
                }
            };
            return Ok(Move::promotion(source.into(), dest.into(), piece.1, promotion, None));
        }
        //TODO castling and en passant
        Ok(Move::normal(source.into(), dest.into(), piece, None))
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
