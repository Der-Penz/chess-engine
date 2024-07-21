use crate::game::{Color, Piece, PieceVariation, Square};

use super::{CastleRights, Move, MoveType};

pub struct DetailedMove {
    piece: Piece,
    source: u8,
    dest: u8,
    move_type: MoveType,
    capture: Option<PieceVariation>,
    check: bool,
    ply_clock: u8,
    castle_rights: CastleRights,
}

impl DetailedMove {
    pub fn new(
        piece: Piece,
        source: u8,
        dest: u8,
        move_type: MoveType,
        capture: Option<PieceVariation>,
        check: bool,
        ply_clock: u8,
        castle_rights: CastleRights,
    ) -> Self {
        DetailedMove {
            piece,
            source,
            dest,
            move_type,
            capture,
            check,
            ply_clock,
            castle_rights,
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
            ply_clock: 0,
            castle_rights: CastleRights::default(),
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

    pub fn ply_clock(&self) -> u8 {
        self.ply_clock
    }

    pub fn castle_rights(&self) -> CastleRights {
        self.castle_rights
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

        write!(
            f,
            "{}: {}->{}{}| ",
            self.piece,
            self.source_sq(),
            self.dest_sq(),
            if self.check { "+" } else { " " }
        )?;
        self.capture.inspect(|c| {
            let _ = write!(f, "⚔:{}| ", Piece(*c, self.color().opposite()));
        });

        write!(f, "Ply: {}| ", self.ply_clock)?;
        write!(f, "{} | ", self.castle_rights)?;
        match self.move_type {
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
