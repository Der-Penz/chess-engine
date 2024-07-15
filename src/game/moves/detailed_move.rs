use crate::game::{ Color, Piece, PieceVariation, Square };

use super::{ CastleType, Move, MoveType, PromotionPiece };

pub struct DetailedMove {
    piece: Piece,
    source: u8,
    dest: u8,
    move_type: MoveType,
    capture: Option<PieceVariation>,
    check: bool,
}

impl DetailedMove {
    pub fn new_normal(
        piece: Piece,
        source: u8,
        dest: u8,
        capture: Option<PieceVariation>,
        check: bool
    ) -> Self {
        DetailedMove {
            piece,
            source,
            dest,
            move_type: MoveType::Normal,
            check,
            capture,
        }
    }

    pub fn new_promotion(
        piece: Piece,
        source: u8,
        dest: u8,
        promotion_piece: PromotionPiece,
        capture: Option<PieceVariation>,
        check: bool
    ) -> Self {
        if piece.0 != PieceVariation::PAWN {
            panic!("Only pawns can be promoted");
        }

        DetailedMove {
            piece,
            source,
            dest,
            move_type: MoveType::Promotion(promotion_piece),
            check,
            capture,
        }
    }
    pub fn new_en_passant(piece: Piece, source: u8, dest: u8, check: bool) -> Self {
        DetailedMove {
            piece,
            source,
            dest,
            move_type: MoveType::EnPassant,
            check,
            capture: Some(PieceVariation::PAWN),
        }
    }

    pub fn new_castle(piece: Piece, source: u8, dest: u8, check: bool) -> Self {
        if piece.0 != PieceVariation::KING {
            panic!("Only kings can castle");
        }

        let castle_type = CastleType::matches_castle(source.into(), dest.into()).expect(
            "Invalid castle move"
        );

        DetailedMove {
            piece,
            source,
            dest,
            move_type: MoveType::Castling(castle_type),
            check,
            capture: None,
        }
    }

    pub fn null() -> Self {
        DetailedMove {
            piece: Default::default(),
            source: std::u8::MAX,
            dest: std::u8::MAX,
            move_type: MoveType::default(),
            check: false,
            capture: None,
        }
    }

    pub fn source(&self) -> u8 {
        self.source
    }

    pub fn source_sq(&self) -> Square {
        Square::from(self.source)
    }

    pub fn dest(&self) -> u8 {
        self.dest
    }

    pub fn dest_sq(&self) -> Square {
        Square::from(self.dest)
    }

    pub fn piece(&self) -> Piece {
        self.piece
    }

    pub fn piece_variation(&self) -> PieceVariation {
        self.piece.0
    }

    pub fn color(&self) -> Color {
        self.piece.1
    }

    pub fn move_type(&self) -> MoveType {
        self.move_type
    }

    pub fn capture(&self) -> Option<PieceVariation> {
        self.capture
    }

    pub fn check(&self) -> bool {
        self.check
    }

    pub fn as_move(&self) -> Move {
        Move::new(self.source, self.dest, self.move_type.into())
    }

    pub fn is_null(&self) -> bool {
        self.source == std::u8::MAX && self.dest == std::u8::MAX
    }

    pub fn valid(&self) -> bool {
        (Square::valid(self.source) && Square::valid(self.dest)) || self.is_null()
    }
}

impl std::fmt::Display for DetailedMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_null() {
            return write!(f, "0000");
        }

        write!(f, "{}: {}->{}{}| ", self.piece(), self.source_sq(), self.dest_sq(), if self.check() {
            "+"
        } else {
            " "
        })?;
        self.capture().inspect(|c| {
            write!(f, "⚔:{}|", Piece(*c, self.color().opposite()));
        });
        match self.move_type() {
            MoveType::EnPassant => write!(f, "En Passant"),
            MoveType::Promotion(p) => write!(f, "Promotion -> {}", Piece(p.into(), self.color())),
            MoveType::Castling(t) => write!(f, "Castling {}", t),
            MoveType::Normal => Ok(()),
        }
    }
}

impl std::fmt::Debug for DetailedMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
