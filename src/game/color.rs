use std::ops::{Index, IndexMut};

#[derive(Debug, FromPrimitive, Clone, Copy, PartialEq)]
pub enum Color {
    WHITE = 0,
    BLACK = 1,
}

impl<T, const N: usize> Index<Color> for [T; N] {
    type Output = T;

    fn index(&self, index: Color) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T, const N: usize> IndexMut<Color> for [T; N] {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        &mut self[index as usize]
    }
}