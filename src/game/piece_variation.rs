use num_traits::FromPrimitive;
use std::ops::{ Index, IndexMut };

#[derive(Debug, FromPrimitive, Clone, Copy, PartialEq)]
pub enum PieceVariation {
    PAWN,
    KNIGHT,
    BISHOP,
    ROOK,
    QUEEN,
    KING,
}

impl PieceVariation {
    pub fn iter() -> impl Iterator<Item = (PieceVariation, usize)> {
        (0..=5usize).map(|i| (PieceVariation::from_usize(i).unwrap(), i))
    }

    pub fn as_char(&self) -> char{
        match self {
            PieceVariation::PAWN => 'p',
            PieceVariation::KNIGHT => 'n',
            PieceVariation::BISHOP => 'b',
            PieceVariation::ROOK => 'r',
            PieceVariation::QUEEN => 'q',
            PieceVariation::KING => 'k',
        }
    }
}

impl<T, const N: usize> Index<PieceVariation> for [T; N] {
    type Output = T;

    fn index(&self, index: PieceVariation) -> &Self::Output {
        &self[index as usize]
    }
}
impl<T, const N: usize> IndexMut<PieceVariation> for [T; N] {
    fn index_mut(&mut self, index: PieceVariation) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl From<char> for PieceVariation {
    fn from(c: char) -> Self {
        match c.to_ascii_lowercase() {
            'p' => PieceVariation::PAWN,
            'n' => PieceVariation::KNIGHT,
            'b' => PieceVariation::BISHOP,
            'r' => PieceVariation::ROOK,
            'q' => PieceVariation::QUEEN,
            'k' => PieceVariation::KING,
            _ => panic!("Invalid char for piece variation"),
        }
    }
}

impl Default for PieceVariation {
    fn default() -> Self {
        PieceVariation::PAWN
    }
}
