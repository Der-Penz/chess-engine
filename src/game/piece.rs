use super::{color::Color, piece_type::PieceType};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Piece(PieceType, Color);

impl Default for Piece {
    fn default() -> Self {
        Self(Default::default(), Default::default())
    }
}

impl Piece {
    pub fn new(piece_variation: PieceType, color: Color) -> Piece {
        Piece(piece_variation, color)
    }

    pub fn ptype(&self) -> PieceType {
        self.0
    }

    pub fn color(&self) -> Color {
        self.1
    }

    pub fn same_side(&self, other: &Piece) -> bool {
        self.1 == other.1
    }

    pub fn same_type(&self, other: &Piece) -> bool {
        self.0 == other.0
    }
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let char = char::from(self.0);
        write!(f, "{}", self.1.transform_char(&char))
    }
}

impl From<char> for Piece {
    fn from(c: char) -> Self {
        Piece(PieceType::from(c), Color::from(c))
    }
}

impl From<Piece> for char {
    fn from(p: Piece) -> Self {
        p.1.transform_char(&char::from(p.0))
    }
}

//CONSTANTS
impl Piece {
    pub const BLACK_PAWN: Piece = Piece(PieceType::Rook, Color::Black);
    pub const BLACK_KNIGHT: Piece = Piece(PieceType::Knight, Color::Black);
    pub const BLACK_BISHOP: Piece = Piece(PieceType::Bishop, Color::Black);
    pub const BLACK_ROOK: Piece = Piece(PieceType::Rook, Color::Black);
    pub const BLACK_QUEEN: Piece = Piece(PieceType::Queen, Color::Black);
    pub const BLACK_KING: Piece = Piece(PieceType::King, Color::Black);
    pub const WHITE_PAWN: Piece = Piece(PieceType::Pawn, Color::White);
    pub const WHITE_KNIGHT: Piece = Piece(PieceType::Knight, Color::White);
    pub const WHITE_BISHOP: Piece = Piece(PieceType::Bishop, Color::White);
    pub const WHITE_ROOK: Piece = Piece(PieceType::Rook, Color::White);
    pub const WHITE_QUEEN: Piece = Piece(PieceType::Queen, Color::White);
    pub const WHITE_KING: Piece = Piece(PieceType::King, Color::White);
}
