use num_traits::FromPrimitive;
use std::ops::{Index, IndexMut};

use crate::attack_pattern::{ATTACK_PATTERN_BISHOP, ATTACK_PATTERN_KING, ATTACK_PATTERN_KNIGHT, ATTACK_PATTERN_PAWN, ATTACK_PATTERN_QUEEN, ATTACK_PATTERN_ROOK};

use super::Color;
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

    pub fn attack_pattern(&self, color: Color) -> [u64; 64] {
        match self {
            PieceVariation::PAWN => ATTACK_PATTERN_PAWN[color],
            PieceVariation::KNIGHT => ATTACK_PATTERN_KNIGHT,
            PieceVariation::BISHOP => ATTACK_PATTERN_BISHOP,
            PieceVariation::ROOK => ATTACK_PATTERN_ROOK,
            PieceVariation::QUEEN => ATTACK_PATTERN_QUEEN,
            PieceVariation::KING => ATTACK_PATTERN_KING,
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