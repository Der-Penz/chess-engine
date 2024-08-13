use super::{color::Color, piece::Piece};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Default for PieceType {
    fn default() -> Self {
        PieceType::Pawn
    }
}

impl PieceType {
    pub fn iter() -> impl Iterator<Item = PieceType> {
        (0..=5usize).map(|i| PieceType::from(i))
    }

    pub fn is_sliding_piece(&self) -> bool {
        matches!(self, PieceType::Bishop | PieceType::Rook | PieceType::Queen)
    }

    pub fn as_colored_piece(&self, color: Color) -> Piece {
        Piece::new(*self, color)
    }
}

impl<T, const N: usize> std::ops::Index<PieceType> for [T; N] {
    type Output = T;

    fn index(&self, index: PieceType) -> &Self::Output {
        &self[index as usize]
    }
}
impl<T, const N: usize> std::ops::IndexMut<PieceType> for [T; N] {
    fn index_mut(&mut self, index: PieceType) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl From<char> for PieceType {
    fn from(c: char) -> Self {
        match c.to_ascii_lowercase() {
            'p' => PieceType::Pawn,
            'n' => PieceType::Knight,
            'b' => PieceType::Bishop,
            'r' => PieceType::Rook,
            'q' => PieceType::Queen,
            'k' => PieceType::King,
            _ => panic!("Invalid char for piece variation"),
        }
    }
}

impl From<PieceType> for char {
    fn from(value: PieceType) -> Self {
        match value {
            PieceType::Pawn => 'p',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Rook => 'r',
            PieceType::Queen => 'q',
            PieceType::King => 'k',
        }
    }
}

impl From<usize> for PieceType {
    fn from(value: usize) -> Self {
        match value {
            0 => PieceType::Pawn,
            1 => PieceType::Knight,
            2 => PieceType::Bishop,
            3 => PieceType::Rook,
            4 => PieceType::Queen,
            5 => PieceType::King,
            _ => panic!("Invalid piece type {}", value),
        }
    }
}

impl std::fmt::Display for PieceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", char::from(*self))
    }
}
